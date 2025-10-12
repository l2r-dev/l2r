use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DestroyItemRequest, Inventory, Item, ItemLocation, ItemsDataQuery, UnequipItems,
        UniqueItem, UpdateType,
    },
    network::packets::server::{GameServerPacket, InventoryUpdate},
    object_id::{ObjectId, ObjectIdManager, QueryByObjectIdMut},
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use smallvec::smallvec;

pub fn destroy_item(
    destroy_request: Trigger<DestroyItemRequest>,
    mut commands: Commands,
    mut items: Query<(Entity, Mut<Item>)>,
    mut inventories: Query<Mut<Inventory>>,
    items_data_query: ItemsDataQuery,
    repo_manager: Res<RepositoryManager>,
    mut unequip_items: EventWriter<UnequipItems>,
    mut object_id_manager: ResMut<ObjectIdManager>,
) -> Result<()> {
    let inventory_entity = destroy_request.target();
    let request = destroy_request.event();

    let mut inventory = inventories.get_mut(inventory_entity)?;

    inventory.get_item(request.item_oid)?;

    let (item_entity, mut item) =
        items.by_object_id_mut(request.item_oid, object_id_manager.as_ref())?;

    let item_id = item.id();
    let item_count = item.count();
    let item_info = items_data_query.get_item_info(item_id)?;
    // Check if item is stackable and if we're destroying the entire stack or just part
    let destroy_full_stack = !item_info.stackable() || request.count >= item_count;

    if destroy_full_stack {
        inventory.remove_item(request.item_oid)?;

        let unique_item = UniqueItem::new(request.item_oid, *item);

        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![unique_item],
                UpdateType::Remove,
            )),
            inventory_entity,
        );

        if matches!(item.location(), ItemLocation::PaperDoll(_)) {
            unequip_items.write(UnequipItems::new(inventory_entity, vec![request.item_oid]));
        }

        // Remove the item entity from the world
        commands.entity(item_entity).despawn();
        object_id_manager.release_id(request.item_oid);

        if !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
            let item_oid = request.item_oid;

            commands.spawn_task(move || async move {
                items_repository.delete_by_id(item_oid).await?;
                Ok(())
            });
        }
    } else {
        // Partial destroy - split the stack
        item.set_count(item_count - request.count);

        let item = *item;
        let item_object_id = request.item_oid;

        if !repo_manager.is_mock() {
            commands.spawn_task(move || async move {
                item.update_count_in_database(item_object_id).await
            });
        }

        let unique_item = UniqueItem::new(item_object_id, item);

        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![unique_item],
                UpdateType::Remove,
            )),
            inventory_entity,
        );
    }

    Ok(())
}
