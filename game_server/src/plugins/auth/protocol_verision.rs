use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    config::GameServerNetworkConfig,
    packets::{
        client::GameClientPacket,
        server::{GameServerPacket, KeyPacket},
    },
    protocol,
    session::{GameServerSession, PacketReceiveParams},
};
use l2r_core::model::session::L2rSession;

pub(crate) struct ClientProtocolVersionPlugin;

impl Plugin for ClientProtocolVersionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut game_sessions: Query<Ref<GameServerSession>>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();

    let session_entity = receive_params.session(&event.connection.id())?;
    let session = game_sessions.get_mut(session_entity)?;

    if let GameClientPacket::ProtocolVersion(ref packet) = event.packet {
        match packet.protocol_version {
            protocol::Version::Unknown => {
                session.disconnect();
            }
            _ => {
                commands.trigger_targets(GameServerPacket::from(KeyPacket::new()), session_entity);
            }
        }
    }
    Ok(())
}
