use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig,
    packets::client::GameClientPacket,
    session::{GameServerSession, PacketReceiveParams},
};
use l2r_core::model::session::L2rSession;

pub(crate) struct NetPingPlugin;
impl Plugin for NetPingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut sessions: Query<Mut<GameServerSession>>,
) -> Result<()> {
    let event = receive.event();

    let GameClientPacket::NetPing = event.packet else {
        return Ok(());
    };
    let session_entity = receive_params.session(&event.connection.id())?;
    let mut session = sessions.get_mut(session_entity)?;
    let ping = session.record_pong().unwrap();
    log::trace!(
        "Pong received from {:?}, ping = {}ms",
        session.id(),
        ping.as_millis()
    );
    Ok(())
}
