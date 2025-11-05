use bevy::prelude::*;
use game_core::{
    items::{
        DestroyItemRequest, Inventory, ItemLocation, ItemsDataAccess, ItemsDataQueryMut,
        UnequipItems, UniqueItem, UpdateType,
    },
    network::packets::server::{GameServerPacket, InventoryUpdate},
};
use smallvec::smallvec;

pub fn destroy_item(
    destroy_request: Trigger<DestroyItemRequest>,
    mut commands: Commands,
    mut inventories: Query<Mut<Inventory>>,
    mut items_data: ItemsDataQueryMut,
    mut unequip_items: EventWriter<UnequipItems>,
) -> Result<()> {
    let inventory_entity = destroy_request.target();
    let request = destroy_request.event();

    let mut inventory = inventories.get_mut(inventory_entity)?;

    inventory.get_item(request.item_oid)?;

    let item_entity = items_data.entity(request.item_oid)?;

    let item = *items_data.item_by_object_id(request.item_oid)?;

    let item_id = item.id();
    let item_count = item.count();
    let item_info = items_data.item_info(item_id)?;
    // Check if item is stackable and if we're destroying the entire stack or just part
    let destroy_full_stack = !item_info.stackable() || request.count >= item_count;

    if destroy_full_stack {
        inventory.remove_item(request.item_oid)?;

        if matches!(item.location(), ItemLocation::PaperDoll(_)) {
            unequip_items.write(UnequipItems::new(inventory_entity, vec![request.item_oid]));
        }

        // Remove the item entity from the world
        commands.entity(item_entity).despawn();
        items_data.object_id_manager.release_id(request.item_oid);

        let unique_item = UniqueItem::new(request.item_oid, item);
        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![unique_item],
                UpdateType::Remove,
            )),
            inventory_entity,
        );
    } else {
        // Partial destroy - split the stack, inventory update will be handled in ItemsPlugin::count_changed
        let mut item = items_data.item_by_object_id_mut(request.item_oid)?;
        item.set_count(item_count - request.count);
    }

    Ok(())
}
