use self::server::{LoginServerPacket, init::InitPacket};
use crate::plugins::network::client::LoginClientPacketPlugin;
use bevy::prelude::*;
use bevy_slinet::server::{NewConnectionEvent, ServerConnection};
use l2r_core::{
    metrics::{Metrics, MetricsAppExt},
    model::session::{L2rSession, ServerSessions, SessionId},
};
use server::LoginServerPacketPlugin;
use std::time::SystemTime;
use strum::Display;

pub mod client;
mod config;
pub mod server;

pub use config::*;

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_slinet::server::ServerPlugin::<
            config::LoginServerNetworkConfig,
        >::bind("0.0.0.0:2106"))
            .add_plugins(LoginClientPacketPlugin)
            .add_plugins(LoginServerPacketPlugin);

        app.insert_resource(bevy_slinet::connection::MaxPacketSize(65535))
            .init_resource::<ServerSessions>();

        // Register metrics
        app.register_counter(LoginNetworkMetric::NewConnections, "New connections count")
            .register_gauge(LoginNetworkMetric::ActiveSessions, "Active sessions count");

        app.add_observer(new_connection);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u32)]
pub enum LoginServerProtocol {
    #[default]
    NewProtocolVersion = 0x0000c621,
    OldProtocolVersion = 192,
}
impl From<LoginServerProtocol> for u32 {
    fn from(version: LoginServerProtocol) -> u32 {
        version as u32
    }
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum LoginNetworkMetric {
    NewConnections,
    ActiveSessions,
}

#[derive(Clone, Component, Debug)]
pub struct LoginServerSession {
    connection: ServerConnection<config::LoginServerNetworkConfig>,
    protocol_version: LoginServerProtocol,
    #[allow(dead_code)]
    start_time: SystemTime,
}
impl L2rSession<config::LoginServerNetworkConfig> for LoginServerSession {
    fn new(connection: ServerConnection<config::LoginServerNetworkConfig>) -> Self {
        // let write current time as start_time
        let start_time = SystemTime::now();
        info!(
            "New LoginServerSession created, for {:?}",
            connection.peer_addr()
        );
        Self {
            start_time,
            protocol_version: LoginServerProtocol::default(),
            connection,
        }
    }

    fn id(&self) -> SessionId {
        self.connection.id().into()
    }

    fn access_level(&self) -> u8 {
        0x00
    }

    fn send(&self, packet: LoginServerPacket) {
        if let Err(e) = self.connection.send(packet) {
            error!("Failed to send packet: {:?}", e);
        }
    }

    fn disconnect(&self) {
        self.connection.disconnect();
    }
}

impl LoginServerSession {
    pub fn _start_time(&self) -> SystemTime {
        self.start_time
    }

    pub fn _protocol_version(&self) -> LoginServerProtocol {
        self.protocol_version
    }
}

fn new_connection(
    new_connection: Trigger<NewConnectionEvent<config::LoginServerNetworkConfig>>,
    metrics: Res<Metrics>,
    mut login_sessions: ResMut<ServerSessions>,
    mut commands: Commands,
) -> Result<()> {
    metrics.counter(LoginNetworkMetric::NewConnections)?.inc();
    metrics.gauge(LoginNetworkMetric::ActiveSessions)?.inc();

    let event = new_connection.event();
    let session = LoginServerSession::new(event.connection.clone());
    let session_id = session.id();
    let init_packet = InitPacket::new(session_id, session.protocol_version);
    let session_entity = commands.spawn(session).id();
    login_sessions.insert(session_id, session_entity);
    commands.trigger_targets(
        LoginServerPacket::from(Box::new(init_packet)),
        session_entity,
    );
    Ok(())
}
