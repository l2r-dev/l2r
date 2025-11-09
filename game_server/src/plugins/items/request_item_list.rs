use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    items::{InventoriesQuery, InventoriesQueryItem, Item, UniqueItem},
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, ItemList},
        },
        session::PacketReceiveParams,
    },
    object_id::{ObjectIdManager, QueryByObjectId},
};

pub(crate) struct RequestItemListPlugin;
impl Plugin for RequestItemListPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    inventories: Query<InventoriesQuery>,
    items: Query<Ref<Item>>,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestItemList = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let InventoriesQueryItem { inventory, .. } = inventories.get(character_entity)?;
        let items_list = inventory
            .iter()
            .filter_map(|object_id| {
                items
                    .by_object_id(*object_id, object_id_manager.as_ref())
                    .ok()
                    .map(|item| UniqueItem::new(*object_id, *item))
            })
            .collect::<Vec<_>>();
        let item_list_packet = ItemList::new(items_list, true);
        commands.trigger_targets(GameServerPacket::from(item_list_packet), character_entity);
    }
    Ok(())
}
