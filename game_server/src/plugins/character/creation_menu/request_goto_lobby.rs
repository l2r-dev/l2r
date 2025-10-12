use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig,
    packets::{client::GameClientPacket, server::SendCharSelectionInfo},
    session::PacketReceiveParams,
};

pub(crate) struct RequestGotoLobbyPlugin;

impl Plugin for RequestGotoLobbyPlugin {
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
    if let GameClientPacket::RequestGotoLobby = event.packet {
        let session_entity = receive_params.session(&event.connection.id())?;
        commands.trigger_targets(SendCharSelectionInfo, session_entity);
    }
    Ok(())
}
