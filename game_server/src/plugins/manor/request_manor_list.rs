use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig, packets::client::GameClientPacket,
    session::PacketReceiveParams,
};

pub(crate) struct RequestManorListPlugin;
impl Plugin for RequestManorListPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestManorList = event.packet {
        let entity = receive_params.character(&event.connection.id())?;
        log::trace!("[{}]RequestManorListPlugin is not implemented yet!", entity);
    }
    Ok(())
}
