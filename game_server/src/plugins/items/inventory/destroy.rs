use bevy::prelude::*;
use game_core::{
    items::{
        DestroyItemRequest, Inventory, Item, ItemLocation, ItemsDataQuery, UnequipItems,
    },
    object_id::{ObjectIdManager, QueryByObjectIdMut},
};

pub fn destroy_item(
    destroy_request: Trigger<DestroyItemRequest>,
    mut commands: Commands,
    mut items: Query<(Entity, Mut<Item>)>,
    mut inventories: Query<Mut<Inventory>>,
    items_data_query: ItemsDataQuery,
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

        if matches!(item.location(), ItemLocation::PaperDoll(_)) {
            unequip_items.write(UnequipItems::new(inventory_entity, vec![request.item_oid]));
        }

        // Remove the item entity from the world
        commands.entity(item_entity).despawn();
        object_id_manager.release_id(request.item_oid);
    } else {
        // Partial destroy - split the stack
        item.set_count(item_count - request.count);
    }

    Ok(())
}
