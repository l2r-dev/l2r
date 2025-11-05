use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DollSlot, ITEMS_OPERATION_STACK, ItemLocationVariant, ItemsDataAccess,
        ItemsDataQuery, ItemsDataQueryMut, ItemsUnEquipped, Kind, PaperDoll, UnequipItems,
        UniqueItem, UpdateType,
    },
    network::packets::server::{
        GameServerPacket, InventoryUpdate, SendCharInfo, SendUserInfo, SystemMessage,
    },
    object_id::ObjectId,
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
    mut paperdolls: Query<Mut<PaperDoll>>,
    mut items_unequipped: EventWriter<ItemsUnEquipped>,
    mut items_data: ItemsDataQueryMut,
) {
    for (event, _event_id) in unequip_events.par_read() {
        let character_entity = event.entity();
        let item_object_ids = event.object_ids().to_vec();

        let Ok(mut paperdoll) = paperdolls.get_mut(character_entity) else {
            return;
        };

        // Phase 1: Plan unequip actions using immutable access
        let mut unequip_plan = Vec::new();

        for item_object_id in item_object_ids {
            let item = match items_data.item_by_object_id(item_object_id) {
                Ok(item) => item,
                Err(_) => continue,
            };

            let item_id = item.id();

            // Check if we need to also unequip ammo from left hand
            let should_unequip_ammo = items_data
                .item_info(item_id)
                .ok()
                .map(|template| template.kind().bow_or_crossbow())
                .unwrap_or(false);

            let ammo_to_unequip = if should_unequip_ammo {
                paperdoll[DollSlot::LeftHand]
            } else {
                None
            };

            unequip_plan.push(UnequipAction {
                item_object_id,
                ammo_to_unequip,
            });
        }

        // Phase 2: Execute unequip actions with mutable access
        let mut unequipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();

        for action in unequip_plan {
            // Unequip main item
            if let Ok(mut item) = items_data.item_by_object_id_mut(action.item_object_id) {
                item.unequip();
                paperdoll.unequip(action.item_object_id);
                unequipped_items.push(action.item_object_id);
            }

            // Unequip ammo if needed
            if let Some(ammo_oid) = action.ammo_to_unequip {
                if let Ok(mut ammo) = items_data.item_by_object_id_mut(ammo_oid) {
                    ammo.unequip();
                    paperdoll.unequip(ammo_oid);
                    unequipped_items.push(ammo_oid);
                }
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

/// Represents a planned unequip action
struct UnequipAction {
    item_object_id: ObjectId,
    ammo_to_unequip: Option<ObjectId>,
}

fn handle_items_unequipped(
    mut unequipped: EventReader<ItemsUnEquipped>,
    mut commands: Commands,
    items_data: ItemsDataQuery,
    repo_manager: Res<RepositoryManager>,
    mut attack_effects: Query<Mut<AttackEffects>>,
    mut stats_modifiers: Query<Mut<StatModifiers>>,
) -> Result<()> {
    for (event, _event_id) in unequipped.par_read() {
        let character_entity = event.entity();

        if event.object_ids().is_empty() {
            continue;
        }

        let item_oids = event.object_ids().to_vec();

        if let Ok(mut stat_modifiers) = stats_modifiers.get_mut(character_entity) {
            for oid in item_oids.iter() {
                let Ok(item_info) = items_data.info_by_object_id(*oid) else {
                    continue;
                };

                if let Some(modifiers) = item_info.stats_modifiers() {
                    stat_modifiers.unmerge(&modifiers);
                }
            }
        }

        if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity) {
            for oid in item_oids.iter() {
                let Ok(item_info) = items_data.info_by_object_id(*oid) else {
                    continue;
                };

                if matches!(item_info.kind(), Kind::Weapon(_)) {
                    attack_effects.set_weapon(Weapon::default());
                }
            }
        }

        let items_list = items_data.unique_items_from_object_ids(item_oids.as_slice());

        // Send message to the client
        send_unequipped_system_messages(commands.reborrow(), character_entity, &items_list);

        if !items_list.is_empty() {
            let inventory_update = InventoryUpdate::new(items_list, UpdateType::Modify);
            commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);
        }

        if !event.skip_db_update() && !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
            commands.spawn_task(move || async move {
                items_repository
                    .update_many(|update| {
                        update
                            .col_expr(
                                items::model::Column::Location,
                                Expr::value(ItemLocationVariant::Inventory),
                            )
                            .filter(items::model::Column::ObjectId.is_in(item_oids))
                    })
                    .await?;

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
