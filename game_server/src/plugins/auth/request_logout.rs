use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig,
    packets::{
        client::GameClientPacket,
        server::{GameServerPacket, LogoutOk},
    },
    session::PacketReceiveParams,
};

pub(crate) struct RequestLogoutPlugin;

impl Plugin for RequestLogoutPlugin {
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
    if let GameClientPacket::RequestLogout = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        commands.trigger_targets(GameServerPacket::from(LogoutOk), character_entity);
    }
    Ok(())
}
