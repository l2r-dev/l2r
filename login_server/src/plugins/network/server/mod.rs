use self::{
    auth_gg::AuthGGResponse, init::InitPacket, login_fail::LoginFail, login_ok::LoginOk,
    play_fail::PlayFail, play_ok::PlayOk, server_list::ServerListResponse,
};
use super::LoginServerSession;
use bevy::prelude::*;
use derive_more::derive::From;
use l2r_core::{
    metrics::Metrics,
    model::session::L2rSession,
    packets::{L2rServerPacket, L2rServerPackets, ServerPacketBuffer, ServerPacketId},
};
use strum::Display;

pub mod auth_gg;
pub mod init;
pub mod login_fail;
pub mod login_ok;
pub mod play_fail;
pub mod play_ok;
pub mod server_list;

pub struct LoginServerPacketPlugin;
impl Plugin for LoginServerPacketPlugin {
    fn build(&self, app: &mut App) {
        use l2r_core::metrics::MetricsAppExt;

        app.add_event::<LoginServerPacket>()
            .register_counter(LoginServerPacketMetric::PacketsSent, "Total packets sent")
            .register_counter(LoginServerPacketMetric::PacketErrors, "Total packet errors")
            .add_observer(send_server_packet);
    }
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum LoginServerPacketMetric {
    PacketsSent,
    PacketErrors,
}

fn send_server_packet(
    packet: Trigger<LoginServerPacket>,
    sessions: Query<&LoginServerSession>,
    metrics: Res<Metrics>,
) -> Result<()> {
    let entity = packet.target();
    let packet = packet.event();
    if let Ok(session) = sessions.get(entity) {
        session.send(packet.clone());
        metrics.counter(LoginServerPacketMetric::PacketsSent)?.inc();
    } else {
        metrics
            .counter(LoginServerPacketMetric::PacketErrors)?
            .inc();
    }
    Ok(())
}

#[derive(Debug)]
pub struct LoginServerPacketCode;
impl LoginServerPacketCode {
    pub const INIT_PACKET: ServerPacketId = ServerPacketId::new(0x00);
    pub const LOGIN_FAIL: ServerPacketId = ServerPacketId::new(0x01);
    pub const AUTH_GG_RESPONSE: ServerPacketId = ServerPacketId::new(0x0b);
    pub const LOGIN_OK: ServerPacketId = ServerPacketId::new(0x03);

    pub const SERVER_LIST_RESPONSE: ServerPacketId = ServerPacketId::new(0x04);
    pub const PLAY_FAIL: ServerPacketId = ServerPacketId::new(0x06);
    pub const PLAY_OK: ServerPacketId = ServerPacketId::new(0x07);
}

#[derive(Clone, Debug, Event, From)]
pub enum LoginServerPacket {
    InitPacket(Box<InitPacket>),
    AuthGGResponse(AuthGGResponse),
    LoginOk(LoginOk),
    LoginFail(LoginFail),
    ServerListResponse(ServerListResponse),
    PlayOk(PlayOk),
    PlayFail(PlayFail),
}
impl Default for LoginServerPacket {
    fn default() -> Self {
        LoginServerPacket::from(PlayFail::ReasonNoMessage)
    }
}

impl L2rServerPackets for LoginServerPacket {
    fn buffer(self) -> ServerPacketBuffer {
        match self {
            LoginServerPacket::InitPacket(s) => s.buffer(),
            LoginServerPacket::AuthGGResponse(s) => s.buffer(),
            LoginServerPacket::LoginOk(s) => s.buffer(),
            LoginServerPacket::LoginFail(s) => s.buffer(),
            LoginServerPacket::ServerListResponse(s) => s.buffer(),
            LoginServerPacket::PlayOk(s) => s.buffer(),
            LoginServerPacket::PlayFail(s) => s.buffer(),
        }
    }
}
