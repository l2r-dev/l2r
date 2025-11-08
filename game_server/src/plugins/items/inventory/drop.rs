use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    active_action::ActiveAction,
    items::{
        self, DropIfPossible, Inventory, Item, ItemInWorld, ItemLocation, ItemMetric,
        ItemsDataAccess, ItemsDataQueryMut, PaperDoll, UniqueItem, UpdateType,
        model::{ActiveModelSetCoordinates, Model},
    },
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{
            DropItem as DropItemPacket, GameServerPacket, InventoryUpdate, SystemMessage,
        },
    },
    object_id::ObjectId,
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    metrics::Metrics,
    plugins::custom_hierarchy::DespawnChildOf,
};
use map::WorldMap;
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use smallvec::smallvec;
use spatial::FlatDistance;
use system_messages::{self, Id, SmParam};

pub struct DropItemPlugin;
impl Plugin for DropItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(drop_if_possible);
    }
}

pub fn drop_if_possible(
    drop_request: Trigger<DropIfPossible>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
    mut items_data: ItemsDataQueryMut,
    mut inventories: Query<
        (
            Ref<ObjectId>,
            Mut<Inventory>,
            Mut<PaperDoll>,
            Ref<Transform>,
        ),
        Without<ActiveAction>,
    >,
    repo_manager: Res<RepositoryManager>,
    metrics: Res<Metrics>,
) -> Result<()> {
    let dropper_entity = drop_request.target();
    let event = drop_request.event();

    let (_, inventory, _, transform) = inventories.get(dropper_entity)?;
    inventory.get_item(event.item_oid)?;
    if transform.translation.flat_distance(&event.location) > 150.0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new_empty(
                Id::YouCannotDiscardSomethingThatFarAwayFromYou,
            )),
            dropper_entity,
        );
        return Ok(());
    }

    let item_entity = items_data.entity(event.item_oid)?;
    let item = items_data.item_by_object_id(event.item_oid)?;
    let item_id = item.id();
    let is_stackable = items_data.item_info(item_id)?.stackable();
    let item_count = item.count();

    let mut dolls_lens = inventories.transmute_lens::<Mut<PaperDoll>>();
    let doll_query = dolls_lens.query();

    if item.equipped() {
        crate::plugins::items::process_unequip(
            dropper_entity,
            event.item_oid,
            commands.reborrow(),
            &mut items_data,
            doll_query,
            true,
            repo_manager.as_ref(),
        )?;
    }

    let (owner_id, mut inventory, _, _) = inventories.get_mut(dropper_entity)?;

    // Check if item is stackable and if we're dropping the entire stack or just part
    let drop_full_stack = !is_stackable || event.count >= item_count;

    let mut item = items_data.item_by_object_id_mut(event.item_oid)?;
    let object_id_to_drop = if drop_full_stack {
        inventory.remove_item(event.item_oid)?;
        item.set_owner(None);
        item.set_location(ItemLocation::World(event.location));

        commands
            .entity(item_entity)
            .insert(ItemInWorld::new(event.location));

        if let Some(region_entity) = world_map.get_by_loc(event.location) {
            commands
                .entity(item_entity)
                .insert(DespawnChildOf(region_entity));
        }

        let unique_item = UniqueItem::new(event.item_oid, *item);

        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![unique_item],
                UpdateType::Remove,
            )),
            dropper_entity,
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

        let updated_inventory_item = UniqueItem::new(event.item_oid, *item);
        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![updated_inventory_item],
                UpdateType::Modify,
            )),
            dropper_entity,
        );
        let item_info = items_data.item_info(item_id)?;
        let new_item = Item::new_with_count(
            item_id,
            event.count,
            ItemLocation::World(event.location),
            item_info,
        );

        let new_object_id = items_data.object_id_manager.next_id();
        let new_unique_item = UniqueItem::new(new_object_id, new_item);

        // Create new item entity for the dropped portion
        let item_info = items_data.item_info(item_id)?;
        let dropped_entity = new_unique_item.spawn(&mut commands, item_info).id();

        if !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;

            commands.spawn_task(move || async move {
                let new_model = Model::from(new_unique_item);
                items_repository.create(&new_model).await?;
                Ok(())
            });
        }

        if let Some(region_entity) = world_map.get_by_loc(event.location) {
            commands
                .entity(dropped_entity)
                .insert(DespawnChildOf(region_entity));
        }

        new_object_id
    };

    metrics.counter(ItemMetric::ItemsDropped)?.inc();

    let item_info = items_data.item_info(item_id)?;

    let drop_item_packet = DropItemPacket::new(
        *owner_id,
        object_id_to_drop,
        item_id,
        event.location,
        item_info.stackable(),
        event.count,
    );

    let message = SystemMessage::new(
        system_messages::Id::YouHaveDroppedS1,
        vec![SmParam::Text(item_info.name().to_string())],
    );

    commands.trigger_targets(GameServerPacket::from(message), dropper_entity);

    commands.trigger_targets(
        ServerPacketBroadcast::new(drop_item_packet.into()),
        dropper_entity,
    );
    Ok(())
}
