use super::LoginClientPacket;
use crate::plugins::{
    accounts::model::Model,
    network::{
        LoginServerNetworkConfig, LoginServerSession,
        server::{
            LoginServerPacket,
            login_fail::{LoginFail, LoginFailReason},
            server_list::ServerListResponse,
        },
    },
    server_manager::GameServerTable,
};
use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use l2r_core::{
    crypt::session_keys::SessionKey,
    model::session::{L2rSession, ServerSessions},
    packets::{ClientPacketBuffer, L2rSerializeError},
};
use std::{convert::TryFrom, fmt::Debug};

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct ServerListRequest {
    session_key_login: i64,
    bytes: Vec<u8>,
}

impl ServerListRequest {
    pub fn validate_key(&self, key: &SessionKey) -> bool {
        self.session_key_login == key.get_login()
    }
}

impl TryFrom<ClientPacketBuffer> for ServerListRequest {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let session_key_login = buffer.i64()?;

        Ok(ServerListRequest {
            session_key_login,
            bytes: buffer.into(),
        })
    }
}

pub(crate) struct ServerListRequestPlugin;
impl Plugin for ServerListRequestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<LoginServerNetworkConfig>>,
    login_sessions: Res<ServerSessions>,
    mut commands: Commands,
    query: Query<(&LoginServerSession, &Model, &SessionKey)>,
    server_table: Res<GameServerTable>,
) -> Result<()> {
    let event = receive.event();
    let LoginClientPacket::ServerList(ref packet) = event.packet else {
        return Ok(());
    };

    let session_entity = login_sessions.by_connection(&event.connection.id())?;
    let (session, account, session_key) = query.get(session_entity)?;
    if !packet.validate_key(session_key) {
        let peer = session.connection.peer_addr();
        let account_name = format!("{:?}", &account);
        log::error!(
            "Account: {}, SessionKey are not valid, possible hack! Disconnecting {}",
            account_name,
            peer
        );
        commands.trigger_targets(
            LoginServerPacket::from(LoginFail::new(
                session.id(),
                LoginFailReason::SystemErrorLoginLater,
            )),
            session_entity,
        );
        session.disconnect();
        commands.entity(session_entity).despawn();
    } else {
        commands.trigger_targets(
            LoginServerPacket::from(
                // TODO: Calculate real last server and char values here
                ServerListResponse::new(server_table.clone(), 9, 0, 0),
            ),
            session_entity,
        );
    }

    Ok(())
}
