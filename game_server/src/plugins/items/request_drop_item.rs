use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    items::DropIfPossible,
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
};

pub(crate) struct RequestDropItemPlugin;

impl Plugin for RequestDropItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestDropItem(ref packet) = event.packet else {
        return Ok(());
    };
    let character_entity = receive_params.character(&event.connection.id())?;
    commands.trigger_targets(
        DropIfPossible {
            item_oid: packet.object_id,
            count: packet.count,
            location: packet.location,
        },
        character_entity,
    );
    Ok(())
}
