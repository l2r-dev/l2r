use super::LoginClientPacket;
use crate::plugins::{
    accounts::model::Model,
    network::{
        LoginServerNetworkConfig, LoginServerSession,
        server::{LoginServerPacket, play_fail::PlayFail, play_ok::PlayOk},
    },
};
use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use l2r_core::{
    crypt::session_keys::SessionKey,
    model::session::{L2rSession, ServerSessions},
    packets::{ClientPacketBuffer, L2rSerializeError},
};
use std::{convert::TryFrom, fmt::Debug};

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct GameServerLoginRequest {
    session_key_login: i64,
    server_id: u8,
}

impl GameServerLoginRequest {
    pub fn validate_key(&self, key: &SessionKey) -> bool {
        self.session_key_login == key.get_login()
    }
}

impl TryFrom<ClientPacketBuffer> for GameServerLoginRequest {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let session_key_login = buffer.i64()?;
        let server_id = buffer.u8()?;

        Ok(GameServerLoginRequest {
            session_key_login,
            server_id,
        })
    }
}

pub(crate) struct GameServerLoginRequestPlugin;
impl Plugin for GameServerLoginRequestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<LoginServerNetworkConfig>>,
    login_sessions: Res<ServerSessions>,
    mut commands: Commands,
    query: Query<(&LoginServerSession, &Model, &SessionKey)>,
) -> Result<()> {
    let event = receive.event();
    if let LoginClientPacket::GameServerLogin(ref packet) = event.packet {
        let session_entity = login_sessions.by_connection(&event.connection.id())?;
        let (session, account, session_key) = query.get(session_entity)?;
        if !packet.validate_key(session_key) {
            warn!("Keys are not valid");
            commands.trigger_targets(
                LoginServerPacket::from(PlayFail::ReasonSystemError),
                session_entity,
            );
            session.disconnect();
            commands.entity(session_entity).despawn();
            return Ok(());
        }

        if account.is_online() {
            warn!("Player is online");
        } else {
            info!("Player is offline, may proceed");
        }
        commands.trigger_targets(
            LoginServerPacket::from(PlayOk::new(*session_key)),
            session_entity,
        );
    }
    Ok(())
}
