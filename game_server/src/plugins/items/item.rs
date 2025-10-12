use bevy::{log, prelude::*};
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{Inventory, Item, ItemLocation, UniqueItem, UpdateType},
    network::packets::server::{GameServerPacket, InventoryUpdate},
    object_id::{ObjectId, ObjectIdManager, QueryByObjectIdMut},
};
use smallvec::smallvec;

pub fn count_changed(
    mut query: Query<(Entity, Ref<ObjectId>, Mut<Item>), Changed<Item>>,
    mut inventories: Query<(Entity, Mut<Inventory>)>,
    mut commands: Commands,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    for (item_entity, item_object_id, mut item) in query.iter_mut() {
        if item.prev_count() == item.count() {
            continue;
        }

        let item_count = item.count();
        if item_count == 0 {
            commands.entity(item_entity).try_despawn();
        } else {
            item.set_prev_count(item_count);
        }

        let inventory_update = match item.location() {
            ItemLocation::PaperDoll(_) | ItemLocation::Inventory => {
                let Some(character_object_id) = item.owner() else {
                    log::warn!("Item {} has no owner, but is in inventory", item.id());
                    continue;
                };

                if let Ok((inventory_entity, mut inventory)) =
                    inventories.by_object_id_mut(character_object_id, object_id_manager.as_ref())
                {
                    if item.count() == 0 {
                        inventory.remove_item(*item_object_id)?;

                        Some((
                            inventory_entity,
                            InventoryUpdate::new(
                                smallvec![UniqueItem::new(*item_object_id, *item)],
                                UpdateType::Remove,
                            ),
                        ))
                    } else {
                        Some((
                            inventory_entity,
                            InventoryUpdate::new(
                                smallvec![UniqueItem::new(*item_object_id, *item)],
                                UpdateType::Modify,
                            ),
                        ))
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(inventory_update) = inventory_update {
            commands.trigger_targets(
                GameServerPacket::from(inventory_update.1),
                inventory_update.0,
            );
        }

        let item = *item;
        let item_object_id = *item_object_id;

        commands
            .spawn_task(move || async move { item.update_count_in_database(item_object_id).await });
    }
    Ok(())
}
