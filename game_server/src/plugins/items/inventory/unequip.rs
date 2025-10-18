use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DollSlot, ITEMS_OPERATION_STACK, Item, ItemLocationVariant, ItemsDataQuery,
        ItemsUnEquipped, Kind, PaperDoll, UnequipItems, UniqueItem, UpdateType,
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

pub struct UnequipItemPlugin;
impl Plugin for UnequipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UnequipItems>()
            .add_event::<ItemsUnEquipped>();

        app.add_systems(
            Update,
            handle_unequip_items.in_set(ItemMechanicsSystems::Unequip),
        )
        .add_systems(
            Update,
            handle_items_unequipped.in_set(ItemMechanicsSystems::Unequip),
        );
    }
}

fn handle_unequip_items(
    mut unequip_events: EventReader<UnequipItems>,
    mut items: Query<Mut<Item>>,
    mut paperdolls: Query<Mut<PaperDoll>>,
    mut items_unequipped: EventWriter<ItemsUnEquipped>,
    items_data_query: ItemsDataQuery,
    object_id_manager: Res<ObjectIdManager>,
) {
    for (event, _event_id) in unequip_events.par_read() {
        let character_entity = event.entity();
        let item_object_ids = event.object_ids().to_vec();

        let Ok(mut paperdoll) = paperdolls.get_mut(character_entity) else {
            return;
        };

        let mut unequipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();

        for item_object_id in item_object_ids {
            let Ok(mut item) = items.by_object_id_mut(item_object_id, object_id_manager.as_ref())
            else {
                continue;
            };

            item.unequip();
            paperdoll.unequip(item_object_id);
            unequipped_items.push(item_object_id);

            if let Ok(template) = items_data_query.get_item_info(item.id())
                && template.kind().bow_or_crossbow()
                    && let Some(left_item) = paperdoll[DollSlot::LeftHand]
                    && let Ok(mut left_uniq_item) =
                        items.by_object_id_mut(left_item.object_id(), object_id_manager.as_ref())
                {
                    left_uniq_item.unequip();
                    paperdoll.unequip(left_item.object_id());
                    unequipped_items.push(left_item.object_id());
                }
        }

        if !unequipped_items.is_empty() {
            items_unequipped.write(ItemsUnEquipped::new(
                character_entity,
                unequipped_items,
                event.skip_db_update(),
            ));
        }
    }
}

fn handle_items_unequipped(
    mut unequipped: EventReader<ItemsUnEquipped>,
    mut commands: Commands,
    items: Query<Ref<Item>>,
    items_data_query: ItemsDataQuery,
    repo_manager: Res<RepositoryManager>,
    mut attack_effects: Query<Mut<AttackEffects>>,
    mut stats_modifiers: Query<Mut<StatModifiers>>,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    for (event, _event_id) in unequipped.par_read() {
        let character_entity = event.entity();
        let items_object_ids_from_event = event.object_ids().to_vec();

        let items_list = items_object_ids_from_event
            .iter()
            .filter_map(|item_object_id| {
                items
                    .by_object_id(*item_object_id, object_id_manager.as_ref())
                    .ok()
                    .map(|item| UniqueItem::new(*item_object_id, *item))
            })
            .collect::<SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]>>();

        if let Ok(mut stat_modifiers) = stats_modifiers.get_mut(character_entity) {
            for unique_item in &items_list {
                let Ok(item_info) = items_data_query.get_item_info(unique_item.item().id()) else {
                    continue;
                };

                if let Some(modifiers) = item_info.stats_modifiers() {
                    stat_modifiers.unmerge(&modifiers);
                }
            }
        }

        if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity) {
            for unique_item in &items_list {
                let Ok(item_info) = items_data_query.get_item_info(unique_item.item().id()) else {
                    continue;
                };

                if matches!(item_info.kind(), Kind::Weapon(_)) {
                    attack_effects.set_weapon(Weapon::default());
                }
            }
        }

        // Send message to the client
        send_unequipped_system_messages(commands.reborrow(), character_entity, &items_list);

        if !items_list.is_empty() {
            let inventory_update = InventoryUpdate::new(items_list, UpdateType::Modify);
            commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);
        }

        if !event.skip_db_update() && !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
            commands.spawn_task(move || async move {
                if !items_object_ids_from_event.is_empty() {
                    items_repository
                        .update_many(|update| {
                            update
                                .col_expr(
                                    items::model::Column::Location,
                                    Expr::value(ItemLocationVariant::Inventory),
                                )
                                .filter(
                                    items::model::Column::ObjectId
                                        .is_in(items_object_ids_from_event),
                                )
                        })
                        .await?;
                }

                Ok(())
            });
        }

        commands.trigger_targets(SendUserInfo, character_entity);
        commands.trigger_targets(SendCharInfo, character_entity)
    }
    Ok(())
}

fn send_unequipped_system_messages(
    mut commands: Commands,
    character_entity: Entity,
    items: &[UniqueItem],
) {
    for unique_item in items.iter() {
        if unique_item.item().enchant_level() > 0 {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new(
                    SmId::TheEquipmentS1S2HasBeenRemoved,
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
                    SmId::S1HasBeenDisarmed,
                    vec![SmParam::Item(unique_item.item().id().into())],
                )),
                character_entity,
            );
        }
    }
}
