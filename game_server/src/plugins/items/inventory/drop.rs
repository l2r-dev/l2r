use bevy::{log, prelude::*};
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    custom_hierarchy::DespawnChildOf,
    items::{
        self, DropIfPossible, DropItemEvent, Inventory, Item, ItemInWorld, ItemLocation,
        ItemMetric, ItemsDataQuery, UnequipItems, UniqueItem, UpdateType,
        model::{ActiveModelSetCoordinates, Model},
    },
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{DropItem, GameServerPacket, InventoryUpdate, SystemMessage},
    },
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId, QueryByObjectIdMut},
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    metrics::Metrics,
};
use map::{WorldMap, id::RegionId};
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use smallvec::smallvec;
use spatial::FlatDistance;
use system_messages::{self, Id, SmParam};

pub fn drop_if_possible(
    drop_request: Trigger<DropIfPossible>,
    mut commands: Commands,
    mut inventories: Query<(Ref<ObjectId>, Mut<Inventory>, Ref<Transform>)>,
    mut unequip_items: EventWriter<UnequipItems>,
    mut drop_item: EventWriter<DropItemEvent>,
    items: Query<&Item>,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    let inventory_entity = drop_request.target();
    let request = drop_request.event();

    let (_owner_id, inventory, transform) = inventories.get_mut(inventory_entity)?;

    inventory.get_item(request.item_oid)?;

    let item = items.by_object_id(request.item_oid, object_id_manager.as_ref())?;

    if transform.translation.flat_distance(&request.location) > 150.0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new_empty(
                Id::YouCannotDiscardSomethingThatFarAwayFromYou,
            )),
            inventory_entity,
        );
        return Ok(());
    }

    // Check if item is equipped and needs to be unequipped first
    if matches!(item.location(), ItemLocation::PaperDoll(_)) {
        unequip_items.write(UnequipItems::new_skip_db(
            inventory_entity,
            vec![request.item_oid],
        ));
    }

    drop_item.write(DropItemEvent::new(
        inventory_entity,
        request.item_oid,
        request.count,
        request.location,
    ));

    Ok(())
}

pub fn drop_item(
    mut drop_events: EventReader<DropItemEvent>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
    items_data_query: ItemsDataQuery,
    mut items: Query<(Entity, Mut<Item>)>,
    mut inventories: Query<(Ref<ObjectId>, Mut<Inventory>)>,
    repo_manager: Res<RepositoryManager>,
    mut object_id_manager: ResMut<ObjectIdManager>,
    metrics: Res<Metrics>,
) -> Result<()> {
    for (event, _event_id) in drop_events.par_read() {
        let inventory_entity = event.entity();
        let (owner_id, mut inventory) = inventories.get_mut(inventory_entity)?;
        if let Err(err) = inventory.get_item(event.item_oid) {
            log::warn!("{}", err.to_string());
            continue;
        }
        let (item_entity, mut item) =
            items.by_object_id_mut(event.item_oid, object_id_manager.as_ref())?;
        let item_id = item.id();
        let item_count = item.count();
        let Ok(item_info) = items_data_query.get_item_info(item_id) else {
            log::warn!(
                "Item info not found for item {} in inventory of entity {}",
                item_id,
                inventory_entity
            );
            continue;
        };

        // Check if item is stackable and if we're dropping the entire stack or just part
        let drop_full_stack = !item_info.stackable() || event.count >= item_count;

        let object_id_to_drop = if drop_full_stack {
            inventory.remove_item(event.item_oid)?;
            item.set_owner(None);
            item.set_location(ItemLocation::World(event.location));

            commands
                .entity(item_entity)
                .insert(ItemInWorld::new(event.location));

            if let Some(region_entity) = world_map.get(&RegionId::from(event.location)) {
                commands
                    .entity(item_entity)
                    .insert(DespawnChildOf(*region_entity));
            }

            let unique_item = UniqueItem::new(event.item_oid, *item);

            commands.trigger_targets(
                GameServerPacket::from(InventoryUpdate::new(
                    smallvec![unique_item],
                    UpdateType::Remove,
                )),
                inventory_entity,
            );

            if !repo_manager.is_mock() {
                let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
                commands.spawn_task(move || async move {
                    let item_model = Model::from(unique_item);

                    let mut active_model = item_model.into_active_model();

                    active_model.set_location(unique_item.item().location());
                    active_model.owner_id = Set(unique_item.item().owner());

                    items_repository.update(&active_model).await?;
                    Ok(())
                });
            }

            event.item_oid
        } else {
            // Partial drop - split the stack, inventory update will be handled in ItemsPlugin::count_changed
            item.set_count(item_count - event.count);

            let new_object_id = object_id_manager.next_id();

            let new_item = Item::new_with_count(
                item_id,
                event.count,
                ItemLocation::World(event.location),
                item_info,
            );

            let new_unique_item = UniqueItem::new(new_object_id, new_item);

            // Create new item entity for the dropped portion
            let dropped_entity = commands
                .spawn((new_unique_item, Transform::from_translation(event.location)))
                .id();

            if !repo_manager.is_mock() {
                let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;

                commands.spawn_task(move || async move {
                    let new_model = Model::from(new_unique_item);
                    items_repository.create(&new_model).await?;
                    Ok(())
                });
            }

            if let Some(region_entity) = world_map.get(&RegionId::from(event.location)) {
                commands
                    .entity(dropped_entity)
                    .insert(DespawnChildOf(*region_entity));
            }

            new_object_id
        };

        // Send UI notifications
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                system_messages::Id::YouHaveDroppedS1,
                vec![SmParam::Text(item_info.name().to_string())],
            )),
            inventory_entity,
        );

        metrics.counter(ItemMetric::ItemsDropped)?.inc();

        let drop_item_packet = DropItem::new(
            *owner_id,
            object_id_to_drop,
            item_id,
            event.location,
            item_info.stackable(),
            event.count,
        );

        commands.trigger_targets(
            ServerPacketBroadcast::new(drop_item_packet.into()),
            inventory_entity,
        );
    }
    Ok(())
}
