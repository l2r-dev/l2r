use super::LoginClientPacket;
use crate::plugins::network::{
    LoginServerNetworkConfig, LoginServerSession,
    server::{LoginServerPacket, auth_gg::AuthGGResponse},
};
use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use l2r_core::{
    model::session::{L2rSession, ServerSessions},
    packets::{ClientPacketBuffer, L2rSerializeError},
};
use std::{convert::TryFrom, fmt::Debug};

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct AuthGGRequest {
    pub session_id: u32,
    pub data1: u32,
    pub data2: u32,
    pub data3: u32,
    pub data4: u32,
    pub bytes: Vec<u8>,
}

impl TryFrom<ClientPacketBuffer> for AuthGGRequest {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let session_id = buffer.u32()?;
        let data1 = buffer.u32()?;
        let data2 = buffer.u32()?;
        let data3 = buffer.u32()?;
        let data4 = buffer.u32()?;

        Ok(Self {
            session_id,
            data1,
            data2,
            data3,
            data4,
            bytes: buffer.into(),
        })
    }
}

pub(crate) struct AuthGGRequestPlugin;
impl Plugin for AuthGGRequestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<LoginServerNetworkConfig>>,
    login_sessions: Res<ServerSessions>,
    mut commands: Commands,
    sessions: Query<Ref<LoginServerSession>>,
) -> Result<()> {
    let event = receive.event();
    if let LoginClientPacket::AuthGG(_) = event.packet {
        let session_entity = login_sessions.by_connection(&event.connection.id())?;
        let session = sessions.get(session_entity)?;
        commands.trigger_targets(
            LoginServerPacket::from(AuthGGResponse::new(session.id())),
            session_entity,
        );
    }
    Ok(())
}
