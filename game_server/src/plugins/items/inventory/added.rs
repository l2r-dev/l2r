use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self,
        model::{ActiveModelSetCoordinates, Model},
        *,
    },
    network::packets::server::{GameServerPacket, InventoryUpdate, SystemMessage},
    object_id::ObjectId,
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    plugins::custom_hierarchy::DespawnChildOf,
};
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use smallvec::smallvec;
use system_messages;

pub struct AddInInventoryPlugin;
impl Plugin for AddInInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_in_inventory);
        app.add_observer(send_item_obtained_message);
    }
}

fn add_in_inventory(
    trigger: Trigger<AddInInventory>,
    mut commands: Commands,
    repo_manager: Res<RepositoryManager>,
    mut inventories: Query<InventoriesQueryMut>,
    mut items_data: ItemsDataQueryMut,
) -> Result<()> {
    let inventory_target = trigger.target();
    let event = trigger.event();
    let item_entity = event.item;
    let new_item_oid = *items_data.object_ids.get(item_entity)?;
    let new_item = *items_data.item(item_entity)?;
    let new_item_id = new_item.id();
    let new_item_count = new_item.count();
    let is_stackable = items_data.item_info(new_item_id)?.stackable();

    // First, try to stack if item is stackable
    if is_stackable {
        let InventoriesQueryMutReadOnlyItem { inventory, .. } =
            inventories.get(inventory_target)?;

        // Try to find a matching item to stack with
        let existing_item_oid = inventory
            .iter()
            .find(|&&oid| {
                items_data
                    .item_by_object_id(oid)
                    .map(|item| item.id() == new_item_id)
                    .unwrap_or(false)
            })
            .copied();

        // Found existing item with same id - stack them together
        if let Some(existing_item_oid) = existing_item_oid {
            let mut existing_item = items_data.item_by_object_id_mut(existing_item_oid)?;
            let existing_item_count = existing_item.count();
            existing_item.set_count(existing_item_count + new_item_count);

            let unique_item = UniqueItem::new(existing_item_oid, *existing_item);

            let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Modify);
            commands.trigger_targets(GameServerPacket::from(inventory_update), inventory_target);

            if !repo_manager.is_mock() {
                let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
                // Update the existing item count
                commands.spawn_task(move || async move {
                    let item_count = unique_item.item().count() as i64;
                    let item_model = Model::from(unique_item);
                    let mut active_model = item_model.into_active_model();
                    active_model.count = Set(item_count);

                    items_repository.update(&active_model).await?;
                    Ok(())
                });
                // Delete the merged item from database
                let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
                commands.spawn_task(move || async move {
                    items_repository.delete_by_id(new_item_oid).await?;
                    Ok(())
                });
            }
            items_data.object_id_manager.release_id(new_item_oid);
            commands.entity(item_entity).try_despawn();

            if !event.silent {
                commands.trigger_targets(
                    ItemObtained {
                        item: new_item_id,
                        count: new_item_count,
                    },
                    inventory_target,
                );
            }
            return Ok(());
        }
    }

    let InventoriesQueryMutItem {
        object_id,
        mut inventory,
        ..
    } = inventories.get_mut(inventory_target)?;

    let mut item = items_data.item_mut(item_entity)?;
    let current_location = item.location();

    // Update item location if it's in the world
    // TODO: other checks when moving from store, trade, etc.
    if matches!(current_location, ItemLocation::World(_)) {
        item.set_location(ItemLocation::Inventory);
        ItemInWorld::move_from_world(commands.entity(item_entity).reborrow());
    }

    item.set_owner(Some(*object_id));
    inventory.insert(new_item_oid);

    let unique_item = UniqueItem::new(new_item_oid, *item);

    let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Add);
    commands.trigger_targets(GameServerPacket::from(inventory_update), inventory_target);

    commands
        .entity(item_entity)
        .insert(DespawnChildOf(inventory_target));

    if !repo_manager.is_mock() {
        let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
        commands.spawn_task(move || async move {
            let item_model = Model::from(unique_item);
            let mut active_model = item_model.into_active_model();

            active_model.count = Set(unique_item.item().count() as i64);
            active_model.owner_id = Set(unique_item.item().owner());
            active_model.set_location(unique_item.item().location());

            items_repository.update(&active_model).await?;
            Ok(())
        });
    }

    if !event.silent {
        commands.trigger_targets(
            ItemObtained {
                item: item.id(),
                count: item.count(),
            },
            inventory_target,
        );
    }

    // Equip ammo if we have a matching bow/crossbow
    if items_data.item_info(new_item_id)?.kind().ammo() {
        let InventoriesQueryMutReadOnlyItem { paper_doll, .. } =
            inventories.get(inventory_target)?;
        if paper_doll.is_ammo_valid_for_weapon(new_item_oid, &items_data) {
            commands.trigger_targets(EquipItem(new_item_oid), inventory_target);
        }
    }

    Ok(())
}

fn send_item_obtained_message(
    trigger: Trigger<ItemObtained>,
    mut commands: Commands,
    items_data: ItemsDataQuery,
) -> Result<()> {
    let ItemObtained { item, count } = trigger.event();
    let item_info = items_data.item_info(*item)?;

    let message = match (*item, *count) {
        // Adena (item ID 57) uses special message
        (id, count) if id == 57.into() => SystemMessage::new(
            system_messages::Id::YouHaveObtainedS1Adena,
            vec![count.into()],
        ),
        // Multiple items of the same type
        (_, count) if count > 1 => SystemMessage::new(
            system_messages::Id::YouHaveObtainedS2S1,
            vec![item_info.name().to_string().into(), count.into()],
        ),
        // Single item
        _ => SystemMessage::new(
            system_messages::Id::YouHaveObtainedS1,
            vec![item_info.name().to_string().into()],
        ),
    };

    commands.trigger_targets(GameServerPacket::from(message), trigger.target());
    Ok(())
}
