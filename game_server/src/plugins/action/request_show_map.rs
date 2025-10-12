use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig,
    packets::{
        client::GameClientPacket,
        server::{GameServerPacket, ShowMap},
    },
    session::PacketReceiveParams,
};

pub(crate) struct RequestShowMapPlugin;
impl Plugin for RequestShowMapPlugin {
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
    let GameClientPacket::RequestShowMap = event.packet else {
        return Ok(());
    };
    let session_entity = receive_params.session(&event.connection.id())?;
    commands.trigger_targets(GameServerPacket::from(ShowMap), session_entity);
    Ok(())
}
