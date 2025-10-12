use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    items::{DestroyItemRequest, Inventory},
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, SystemMessage},
        },
        session::PacketReceiveParams,
    },
};
use system_messages::Id;

pub(crate) struct RequestDestroyItemPlugin;

impl Plugin for RequestDestroyItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    characters: Query<Ref<Inventory>>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestDestroyItem(ref packet) = event.packet else {
        return Ok(());
    };
    let entity = receive_params.character(&event.connection.id())?;
    let inventory = characters.get(entity)?;
    if inventory.get_item(packet.object_id).is_err() {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new_empty(Id::IncorrectItem)),
            entity,
        );
        return Ok(());
    }
    commands.trigger_targets(
        DestroyItemRequest {
            item_oid: packet.object_id,
            count: packet.count,
        },
        entity,
    );
    Ok(())
}
