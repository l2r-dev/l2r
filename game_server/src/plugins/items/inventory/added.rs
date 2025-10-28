use avian3d::prelude::Collider;
use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncCommandsExtension};
use game_core::{
    custom_hierarchy::{DespawnChildOf, DespawnChildren},
    items::{
        self,
        model::{ActiveModelSetCoordinates, Model},
        *,
    },
    network::packets::server::{
        GameServerPacket, GameServerPackets, InventoryUpdate, SystemMessage,
    },
    object_id::{ObjectId, ObjectIdManager, QueryByObjectIdMut},
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use smallvec::SmallVec;
use system_messages;

pub(super) fn add_in_inventory(
    trigger: Trigger<AddInInventory>,
    mut commands: Commands,
    items_data_query: ItemsDataQuery,
    items: Query<(Ref<ObjectId>, Mut<Item>)>,
) -> Result<()> {
    let inventory_target = trigger.target();
    let event = trigger.event();
    let silent = event.silent;

    let mut stackable_items = SmallVec::<[Entity; ITEMS_OPERATION_STACK]>::new();
    let mut non_stackable_items = SmallVec::<[Entity; ITEMS_OPERATION_STACK]>::new();

    for &new_item_entity in event.items.iter() {
        let (_, new_item) = items.get(new_item_entity)?;

        let new_item_info = items_data_query.get_item_info(new_item.id())?;

        if new_item_info.stackable() {
            stackable_items.push(new_item_entity);
        } else {
            non_stackable_items.push(new_item_entity);
        }
    }

    if !stackable_items.is_empty() {
        let event = if silent {
            AddStackable::new_silent(stackable_items)
        } else {
            AddStackable::new(stackable_items)
        };
        commands.trigger_targets(event, inventory_target);
    }

    if !non_stackable_items.is_empty() {
        let event = if silent {
            AddNonStackable::new_silent(non_stackable_items)
        } else {
            AddNonStackable::new(non_stackable_items)
        };

        commands.trigger_targets(event, inventory_target);
    }
    Ok(())
}

async fn update_items_in_database(
    items_repository: ItemsRepository,
    db_items_to_update: SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]>,
) -> Result<(), AccessError> {
    let mut active_models =
        SmallVec::<[_; ITEMS_OPERATION_STACK]>::with_capacity(db_items_to_update.len());

    for unique_item in db_items_to_update {
        let item_model = Model::from(unique_item);
        let mut active_model = item_model.into_active_model();

        active_model.count = Set(unique_item.item().count() as i64);
        active_model.owner_id = Set(unique_item.item().owner());
        active_model.set_location(unique_item.item().location());

        log::trace!("Updating item in database: {:?}", active_model);

        active_models.push(active_model);
    }

    items_repository
        .update_in_transaction(&active_models)
        .await?;
    Ok(())
}

pub(super) fn add_non_stackable(
    added_non_stackable: Trigger<AddNonStackable>,
    mut commands: Commands,
    repo_manager: Res<RepositoryManager>,
    mut items: Query<(Ref<ObjectId>, Mut<Item>)>,
    mut inventories: Query<(Ref<ObjectId>, Mut<Inventory>)>,
    items_data_query: ItemsDataQuery,
) -> Result<()> {
    let inventory_target = added_non_stackable.target();
    let event = added_non_stackable.event();
    let new_items_to_add = &event.items;
    let silent = event.silent;

    let (owner_id, mut inventory) = match inventories.get_mut(inventory_target) {
        Ok(inv) => inv,
        Err(_) => {
            log::warn!(
                "No inventory component found for entity {}",
                inventory_target
            );
            return Ok(());
        }
    };

    let mut items_to_add = SmallVec::<[UniqueItem; ITEMS_OPERATION_STACK]>::new();
    let mut items_to_update_in_db = SmallVec::<[UniqueItem; ITEMS_OPERATION_STACK]>::new();

    for &new_item_entity in new_items_to_add.iter() {
        let (new_item_oid, mut new_item) = match items.get_mut(new_item_entity) {
            Ok(item) => item,
            Err(_) => return Ok(()),
        };

        let current_owner = new_item.owner();
        let current_location = new_item.location();

        if matches!(current_location, ItemLocation::World(_)) {
            new_item.set_location(ItemLocation::Inventory);

            commands
                .entity(new_item_entity)
                .remove::<Transform>()
                .remove::<GlobalTransform>()
                .remove::<Collider>()
                .remove::<DespawnChildren>();
        }

        new_item.set_owner(Some(*owner_id));

        inventory.insert(*new_item_oid);

        let unique_item = UniqueItem::new(*new_item_oid, *new_item);
        items_to_add.push(unique_item);

        // Only update in DB if owner changed or location is not Inventory or PaperDoll
        if current_owner != Some(*owner_id)
            || !(matches!(current_location, ItemLocation::Inventory)
                || matches!(current_location, ItemLocation::PaperDoll(_)))
        {
            items_to_update_in_db.push(unique_item);
        }

        commands
            .entity(new_item_entity)
            .insert(DespawnChildOf(inventory_target));
    }

    if !items_to_add.is_empty() {
        let inventory_update = InventoryUpdate::new(items_to_add.clone(), UpdateType::Add);
        commands.trigger_targets(GameServerPacket::from(inventory_update), inventory_target);

        // Only send system messages if not in silent mode
        if !silent {
            let mut system_messages = SmallVec::<[GameServerPacket; ITEMS_OPERATION_STACK]>::new();

            for item in items_to_add.iter() {
                let item_info = items_data_query.get_item_info(item.item().id())?;
                let system_message =
                    create_item_obtained_message(item.item().id(), item.item().count(), item_info);
                system_messages.push(system_message);
            }

            commands.trigger_targets(
                GameServerPackets::from(system_messages.to_vec()),
                inventory_target,
            );
        }
    }

    if repo_manager.is_mock() {
        return Ok(());
    }

    let items_repository = repo_manager
        .typed::<ObjectId, items::model::Entity>()
        .map_err(|e| {
            log::error!("Failed to get items repository: {:?}", e);
            e
        })?;

    commands.spawn_task(move || async move {
        update_items_in_database(items_repository, items_to_update_in_db).await
    });

    Ok(())
}

pub(super) fn add_stackable(
    added_stackable: Trigger<AddStackable>,
    mut commands: Commands,
    inventories: CharacterInventories,
    mut items: Query<(Ref<ObjectId>, Mut<Item>)>,
    mut object_id_manager: ResMut<ObjectIdManager>,
    repo_manager: Res<RepositoryManager>,
    items_data_query: ItemsDataQuery,
) -> Result<()> {
    let inventory_target = added_stackable.target();
    let event = added_stackable.event();
    let new_items_to_add = event.items.clone();
    let silent = event.silent;

    let inventory = inventories.get(inventory_target)?;

    let mut db_deletes = SmallVec::<[ObjectId; ITEMS_OPERATION_STACK]>::new();
    let mut db_updates = SmallVec::<[UniqueItem; ITEMS_OPERATION_STACK]>::new();
    let mut inventory_updates = SmallVec::<[UniqueItem; ITEMS_OPERATION_STACK]>::new();
    let mut added_counts = SmallVec::<[(items::Id, u64); ITEMS_OPERATION_STACK]>::new();
    let mut new_in_inventory = SmallVec::<[Entity; ITEMS_OPERATION_STACK]>::new();

    for &new_item_entity in new_items_to_add.iter() {
        let (new_item_oid, new_item) = match items.get(new_item_entity) {
            Ok(item) => (*item.0, *item.1),
            Err(_) => continue,
        };

        let mut found_match = false;

        // Try to find a matching item to stack with
        for &existing_item_oid in inventory.iter() {
            if let Ok((_, mut existing_item)) =
                items.by_object_id_mut(existing_item_oid, object_id_manager.as_ref())
                && existing_item.id() == new_item.id()
            {
                // Found a match - update the existing item
                let existing_item_count = existing_item.count();
                let added_count = new_item.count();
                existing_item.set_count(existing_item_count + added_count);

                let existing_unique_item = UniqueItem::new(existing_item_oid, *existing_item);
                db_updates.push(existing_unique_item);
                inventory_updates.push(existing_unique_item);
                added_counts.push((new_item.id(), added_count));

                db_deletes.push(new_item_oid);
                commands.entity(new_item_entity).try_despawn();

                found_match = true;
                break;
            }
        }

        // If no matching item was found inventory, consider it like a new item
        // and add it to the inventory
        if !found_match {
            if let Ok((_, mut item)) = items.get_mut(new_item_entity)
                && matches!(item.location(), ItemLocation::World(_))
            {
                item.set_location(ItemLocation::Inventory);
                added_counts.push((item.id(), item.count()));
                commands
                    .entity(new_item_entity)
                    .remove::<Transform>()
                    .remove::<GlobalTransform>()
                    .remove::<DespawnChildren>();
            }

            new_in_inventory.push(new_item_entity);
        }
    }

    // Send inventory updates for stacked items
    if !inventory_updates.is_empty() {
        let inventory_update = InventoryUpdate::new(inventory_updates.clone(), UpdateType::Modify);
        commands.trigger_targets(GameServerPacket::from(inventory_update), inventory_target);

        // Only send system messages if not in silent mode
        if !silent {
            let mut system_messages = SmallVec::<[GameServerPacket; ITEMS_OPERATION_STACK]>::new();

            for (item_id, added_count) in added_counts.iter() {
                let item_info = items_data_query.get_item_info(*item_id)?;
                let system_message =
                    create_item_obtained_message(*item_id, *added_count, item_info);
                system_messages.push(system_message);
            }

            commands.trigger_targets(
                GameServerPackets::from(system_messages.to_vec()),
                inventory_target,
            );
        }
    }

    // Trigger AddNonStackable for items that are new in inventory
    if !new_in_inventory.is_empty() {
        let event = if silent {
            AddNonStackable::new_silent(new_in_inventory)
        } else {
            AddNonStackable::new(new_in_inventory)
        };
        commands.trigger_targets(event, inventory_target);
    }

    if repo_manager.is_mock() {
        return Ok(());
    }

    let items_repository = repo_manager
        .typed::<ObjectId, items::model::Entity>()
        .map_err(|e| {
            log::error!("Failed to get items repository: {:?}", e);
            e
        })?;

    if !db_updates.is_empty() {
        let items_repo = items_repository.clone();
        commands.spawn_task(move || async move {
            update_items_in_database(items_repo, db_updates).await
        });
    }

    // Delete items from db that were merged
    if !db_deletes.is_empty() {
        let items_repo = items_repository.clone();
        object_id_manager.release_ids(db_deletes.as_slice());
        commands.spawn_task(move || async move {
            items_repo.delete_by_ids(db_deletes).await?;
            Ok(())
        });
    }

    Ok(())
}

fn create_item_obtained_message(
    item_id: items::Id,
    count: u64,
    item_info: &ItemInfo,
) -> GameServerPacket {
    if item_id == 57.into() {
        // Adena (item ID 57) uses special message
        GameServerPacket::from(SystemMessage::new(
            system_messages::Id::YouHaveObtainedS1Adena,
            vec![count.into()],
        ))
    } else if count > 1 {
        // Multiple items of the same type
        GameServerPacket::from(SystemMessage::new(
            system_messages::Id::YouHaveObtainedS2S1,
            vec![count.into(), item_info.name().to_string().into()],
        ))
    } else {
        // Single item
        GameServerPacket::from(SystemMessage::new(
            system_messages::Id::YouHaveObtainedS1,
            vec![item_info.name().to_string().into()],
        ))
    }
}
