use super::{
    config::GameServerNetworkConfig, packets::server::GameServerPacket, protocol::Version,
};
use crate::{character, network::packets::server::GameServerPacketKind};
use bevy::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_slinet::{connection::ConnectionId, server::ServerConnection};
use l2r_core::model::session::{L2rSession, ServerSessions, SessionId};
use std::time::{Duration, Instant, SystemTime};

#[derive(Clone, Component, Debug)]
pub struct GameServerSession {
    connection: ServerConnection<GameServerNetworkConfig>,
    protocol_version: Version,
    start_time: SystemTime,
    last_ping_sent: Option<Instant>,
    current_ping: Option<Duration>,
}

impl PartialEq for GameServerSession {
    fn eq(&self, other: &Self) -> bool {
        self.connection.id() == other.connection.id()
    }
}

impl GameServerSession {
    const TICK_RATE_ADJUSTMENT: Duration =
        Duration::from_millis(1000 / crate::consts::UPDATE_TICK_RATE as u64);

    pub fn start_ping(&mut self) {
        self.last_ping_sent = Some(Instant::now());
    }

    pub fn record_pong(&mut self) -> Option<Duration> {
        let sent_time = self.last_ping_sent.take()?;
        let ping_time = Instant::now().duration_since(sent_time);
        let adjusted_ping_time = ping_time
            .checked_sub(Self::TICK_RATE_ADJUSTMENT)
            .unwrap_or(ping_time);
        self.current_ping = Some(adjusted_ping_time);
        Some(adjusted_ping_time)
    }

    pub fn current_ping(&self) -> Option<Duration> {
        self.current_ping
    }

    pub fn protocol_version(&self) -> Version {
        self.protocol_version
    }

    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }
}

#[derive(SystemParam)]
pub struct PacketReceiveParams<'w, 's> {
    sessions: Res<'w, ServerSessions>,
    character_tables: Query<'w, 's, Ref<'static, character::Table>>,
}

impl<'w, 's> PacketReceiveParams<'w, 's> {
    pub fn character(&self, connection_id: &ConnectionId) -> Result<Entity> {
        self.sessions
            .get_character_entity(connection_id, &self.character_tables)
    }
    pub fn session(&self, connection_id: &ConnectionId) -> Result<Entity> {
        self.sessions.by_connection(connection_id)
    }
    pub fn character_table(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<Ref<'_, character::Table>> {
        let session_entity = self.sessions.by_connection(connection_id)?;
        Ok(self.character_tables.get(session_entity)?)
    }
}

pub trait GetCharEntity {
    fn get_character_entity(
        &self,
        connection_id: &ConnectionId,
        character_tables: &Query<Ref<character::Table>>,
    ) -> Result<Entity>;
}

impl GetCharEntity for ServerSessions {
    fn get_character_entity(
        &self,
        connection_id: &ConnectionId,
        character_tables: &Query<Ref<character::Table>>,
    ) -> Result<Entity> {
        let session_entity = self.by_connection(connection_id)?;
        let table = character_tables.get(session_entity)?;
        table.character()
    }
}

impl L2rSession<GameServerNetworkConfig> for GameServerSession {
    fn new(connection: ServerConnection<GameServerNetworkConfig>) -> Self {
        let protocol_version = Version::HighFiveUpdate3;
        let start_time = SystemTime::now();
        info!(
            "New GameServerSession created, for {:?}",
            connection.peer_addr()
        );

        Self {
            start_time,
            protocol_version,
            connection,
            last_ping_sent: None,
            current_ping: None,
        }
    }

    fn access_level(&self) -> u8 {
        0x00
    }

    fn id(&self) -> SessionId {
        self.connection.id().into()
    }

    fn send(&self, packet: GameServerPacket) {
        let packet_kind = GameServerPacketKind::from(&packet);
        match self.connection.send(packet) {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "Failed to send packet: {} ({:?}), for session: {:?}",
                    packet_kind,
                    e,
                    self.id()
                );
            }
        }
    }

    fn disconnect(&self) {
        self.connection.disconnect();
    }
}
