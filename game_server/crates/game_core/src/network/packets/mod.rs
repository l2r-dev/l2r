pub mod client;
pub mod server;

use strum::Display;

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum GameServerPacketMetric {
    PacketsSent,
}
