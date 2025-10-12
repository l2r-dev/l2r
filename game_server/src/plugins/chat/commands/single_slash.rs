use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig, packets::client::GameClientPacket,
    session::PacketReceiveParams,
};

pub(crate) struct SingleSlashCommandPlugin;
impl Plugin for SingleSlashCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
) {
    let event = receive.event();
    if let GameClientPacket::SingleSlashCommand(ref packet) = event.packet
        && let Ok(character_entity) = receive_params.character(&event.connection.id())
    {
        commands.trigger_targets(packet.command, character_entity);
    }
}
