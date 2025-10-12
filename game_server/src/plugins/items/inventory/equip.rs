use bevy::{log, prelude::*};
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, EquipItems, ITEMS_OPERATION_STACK, Item, ItemLocationVariant, ItemsDataQuery,
        ItemsEquipped, ItemsUnEquipped, Kind, PaperDoll, UniqueItem, UpdateType, model,
    },
    network::packets::server::{
        GameServerPacket, InventoryUpdate, SendCharInfo, SendUserInfo, SystemMessage,
    },
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId, QueryByObjectIdMut},
    stats::{AttackEffects, StatModifiers, Weapon},
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::{ColumnTrait, QueryFilter, prelude::Expr};
use smallvec::SmallVec;
use state::ItemMechanicsSystems;
use system_messages::{Id as SmId, SmParam};

pub struct EquipItemPlugin;
impl Plugin for EquipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EquipItems>().add_event::<ItemsEquipped>();

        app.add_systems(
            Update,
            handle_equip_items.in_set(ItemMechanicsSystems::Equip),
        )
        .add_systems(
            Update,
            handle_items_equipped.in_set(ItemMechanicsSystems::Equip),
        );
    }
}

fn handle_equip_items(
    mut equip: EventReader<EquipItems>,
    mut items_equipped: EventWriter<ItemsEquipped>,
    mut items_unequipped: EventWriter<ItemsUnEquipped>,
    mut items: Query<(Ref<ObjectId>, Mut<Item>)>,
    items_data_query: ItemsDataQuery,
    mut characters: Query<(Mut<PaperDoll>, Mut<StatModifiers>)>,

    object_id_manager: Res<ObjectIdManager>,
) {
    for (event, _event_id) in equip.par_read() {
        let character_entity = event.entity();
        let item_object_ids = event.object_ids().to_vec();

        let Ok((mut paperdoll, mut stat_modifiers)) = characters.get_mut(character_entity) else {
            continue;
        };

        let mut equipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();
        let mut unequipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();

        for item_object_id in item_object_ids {
            let Ok((_, mut item)) =
                items.by_object_id_mut(item_object_id, object_id_manager.as_ref())
            else {
                log::warn!(
                    "EquipItems: No item found for object id {:?}",
                    item_object_id
                );
                continue;
            };

            let Ok(item_info) = items_data_query.get_item_info(item.id()) else {
                log::warn!("EquipItems: No item info found for item id {:?}", item.id());
                continue;
            };

            let Some(bodypart) = item_info.bodypart() else {
                log::warn!("EquipItems: No bodypart found for item id {:?}", item.id());
                continue;
            };

            let (doll_slot, previous_items) =
                paperdoll.equip(bodypart, Some(UniqueItem::new(item_object_id, *item)));

            item.equip(doll_slot);

            equipped_items.push(item_object_id);

            for previous in previous_items.into_iter().flatten() {
                let prev_oid = previous.object_id();
                let (_, mut prev_item) =
                    match items.by_object_id_mut(prev_oid, object_id_manager.as_ref()) {
                        Ok(item) => item,
                        Err(_) => continue,
                    };
                paperdoll.unequip(prev_oid);
                prev_item.unequip();
                unequipped_items.push(prev_oid);
            }

            if let Some(stats) = item_info.stats_modifiers() {
                stat_modifiers.merge(&stats);
            }
        }

        if !equipped_items.is_empty() {
            items_equipped.write(ItemsEquipped::new(character_entity, equipped_items));
        }

        if !unequipped_items.is_empty() {
            items_unequipped.write(ItemsUnEquipped::new(
                character_entity,
                unequipped_items,
                false,
            ));
        }
    }
}

fn handle_items_equipped(
    mut equipped: EventReader<ItemsEquipped>,
    mut commands: Commands,
    items: Query<Ref<Item>>,
    repo_manager: Res<RepositoryManager>,
    items_data_query: ItemsDataQuery,
    mut attack_effects: Query<Mut<AttackEffects>>,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    for (event, _event_id) in equipped.par_read() {
        let character_entity = event.entity();
        let items_object_ids = event.object_ids().to_vec();

        let Ok(mut attack_effects) = attack_effects.get_mut(character_entity) else {
            continue;
        };

        for item_object_id in items_object_ids.iter() {
            let item_id = match items.by_object_id(*item_object_id, object_id_manager.as_ref()) {
                Ok(item) => item.id(),
                Err(_) => continue,
            };

            let Ok(item_info) = items_data_query.get_item_info(item_id) else {
                continue;
            };

            if let Kind::Weapon(weapon) = item_info.kind() {
                let weapon_effect = Weapon::from(weapon.kind);
                attack_effects.set_weapon(weapon_effect);
            };
        }

        let items_list = items_object_ids
            .iter()
            .filter_map(|item_object_id| {
                items
                    .by_object_id(*item_object_id, object_id_manager.as_ref())
                    .ok()
                    .map(|item| UniqueItem::new(*item_object_id, *item))
            })
            .collect::<SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]>>();

        // Send system messages for equipped items
        send_system_messages(commands.reborrow(), character_entity, &items_list);

        let inventory_update = InventoryUpdate::new(items_list, UpdateType::Modify);

        commands.trigger_targets(SendUserInfo, character_entity);
        commands.trigger_targets(SendCharInfo, character_entity);

        commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);

        if repo_manager.is_mock() {
            continue;
        }
        let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
        commands.spawn_task(move || async move {
            if !items_object_ids.is_empty() {
                items_repository
                    .update_many(|update| {
                        update
                            .col_expr(
                                model::Column::Location,
                                Expr::value(ItemLocationVariant::PaperDoll),
                            )
                            .filter(model::Column::ObjectId.is_in(items_object_ids))
                    })
                    .await?;
            }
            Ok(())
        });
    }
    Ok(())
}

fn send_system_messages(mut commands: Commands, character_entity: Entity, items: &[UniqueItem]) {
    for unique_item in items.iter() {
        if unique_item.item().enchant_level() > 0 {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new(
                    SmId::EquippedS1S2,
                    vec![
                        SmParam::Number(unique_item.item().enchant_level().into()),
                        SmParam::Item(unique_item.item().id().into()),
                    ],
                )),
                character_entity,
            );
        } else {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new(
                    SmId::YouHaveEquippedYourS1,
                    vec![SmParam::Item(unique_item.item().id().into())],
                )),
                character_entity,
            );
        }
    }
}
