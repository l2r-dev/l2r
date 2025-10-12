pub mod broadcast;
pub mod config;
pub mod packets;
pub mod protocol;
pub mod session;

use strum::Display;

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum GameNetworkMetric {
    PacketsReceived,
    NewConnections,
    ActiveSessions,
}
