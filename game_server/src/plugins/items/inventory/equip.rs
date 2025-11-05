use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, BodyPart, DollSlot, EquipItems, ITEMS_OPERATION_STACK, Inventory,
        ItemLocationVariant, ItemsDataAccess, ItemsDataQuery, ItemsDataQueryMut, ItemsEquipped,
        ItemsUnEquipped, Kind, PaperDoll, UniqueItem, UpdateType, model,
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
    mut characters: Query<(Mut<PaperDoll>, Mut<StatModifiers>, Ref<Inventory>)>,
    mut items_query: ItemsDataQueryMut,
) {
    for (event, _event_id) in equip.par_read() {
        let character_entity = event.entity();
        let item_object_ids = event.object_ids().to_vec();

        let Ok((mut paperdoll, mut stat_modifiers, inventory)) =
            characters.get_mut(character_entity)
        else {
            continue;
        };

        // Phase 1: Validate and plan equipment changes using immutable access
        let mut equip_plan = Vec::new();

        for item_object_id in item_object_ids {
            let item = match items_query.item_by_object_id(item_object_id) {
                Ok(item) => item,
                Err(_) => {
                    warn!(
                        "EquipItems: No item found for object id {:?}",
                        item_object_id
                    );
                    continue;
                }
            };

            let item_info = match items_query.item_info(item.id()) {
                Ok(info) => info,
                Err(_) => {
                    warn!("EquipItems: No item info found for item id {:?}", item.id());
                    continue;
                }
            };

            let bodypart = match item_info.bodypart() {
                Some(bp) => bp,
                None => {
                    warn!("EquipItems: No bodypart found for item id {:?}", item.id());
                    continue;
                }
            };

            // Validate ammo equipping to left hand
            if bodypart == BodyPart::LeftHand && item_info.kind().ammo() {
                let valid_ammo = paperdoll[DollSlot::RightHand]
                    .and_then(|rh_oid| items_query.info_by_object_id(rh_oid).ok())
                    .map(|rh_info| rh_info.ammo_matches(item_info))
                    .unwrap_or(false);

                if !valid_ammo {
                    continue;
                }
            }

            // Determine which slot and what gets unequipped
            let (doll_slot, previous_items) = match paperdoll.equip(item_object_id, &items_query) {
                Some(result) => result,
                None => continue,
            };

            // Check if we need to auto-equip ammo for bow/crossbow
            let ammo_to_equip = if bodypart == BodyPart::BothHand
                && paperdoll[DollSlot::LeftHand].is_none()
                && item_info.kind().bow_or_crossbow()
            {
                find_matching_ammo(&items_query, &inventory, item_info)
            } else {
                None
            };

            equip_plan.push(EquipAction {
                item_object_id,
                doll_slot,
                previous_items,
                ammo_to_equip,
                stats: item_info.stats_modifiers(),
            });
        }

        // Phase 2: Execute equipment changes with mutable access
        let mut equipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();
        let mut unequipped_items = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();

        for action in equip_plan {
            // Equip main item
            if let Ok(mut item) = items_query.item_by_object_id_mut(action.item_object_id) {
                item.equip(action.doll_slot);
                equipped_items.push(action.item_object_id);
            }

            // Equip ammo if needed
            if let Some(ammo_oid) = action.ammo_to_equip {
                let prev_item = paperdoll.equip_without_validations(DollSlot::LeftHand, ammo_oid);

                if let Ok(mut ammo) = items_query.item_by_object_id_mut(ammo_oid) {
                    ammo.equip(DollSlot::LeftHand);
                    equipped_items.push(ammo_oid);
                }

                // If there was a previous item in left hand, unequip it
                if let Some(prev_oid) = prev_item {
                    if let Ok(mut prev) = items_query.item_by_object_id_mut(prev_oid) {
                        prev.unequip();
                        unequipped_items.push(prev_oid);
                    }
                }
            }

            // Unequip previous items
            for previous_oid in action.previous_items {
                if let Ok(mut prev_item) = items_query.item_by_object_id_mut(previous_oid) {
                    paperdoll.unequip(previous_oid);
                    prev_item.unequip();
                    unequipped_items.push(previous_oid);
                }
            }

            // Apply stat modifiers
            if let Some(stats) = action.stats {
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

/// Represents a planned equipment action
struct EquipAction {
    item_object_id: ObjectId,
    doll_slot: DollSlot,
    previous_items: Vec<ObjectId>,
    ammo_to_equip: Option<ObjectId>,
    stats: Option<StatModifiers>,
}

/// Find matching ammo in inventory for the given weapon
fn find_matching_ammo(
    items_query: &impl ItemsDataAccess,
    inventory: &Inventory,
    weapon_info: &items::ItemInfo,
) -> Option<ObjectId> {
    for ammo_oid in inventory.iter() {
        if let Ok(ammo_info) = items_query.info_by_object_id(*ammo_oid) {
            if weapon_info.ammo_matches(ammo_info) {
                return Some(*ammo_oid);
            }
        }
    }
    None
}

fn handle_items_equipped(
    mut equipped: EventReader<ItemsEquipped>,
    mut commands: Commands,
    repo_manager: Res<RepositoryManager>,
    items_query: ItemsDataQuery,
    mut attack_effects: Query<Mut<AttackEffects>>,
) -> Result<()> {
    for (event, _event_id) in equipped.par_read() {
        let character_entity = event.entity();
        let items_object_ids = event.object_ids().to_vec();

        let Ok(mut attack_effects) = attack_effects.get_mut(character_entity) else {
            continue;
        };

        for item_object_id in items_object_ids.iter() {
            let Ok(item_info) = items_query.info_by_object_id(*item_object_id) else {
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
                items_query
                    .item_by_object_id(*item_object_id)
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
