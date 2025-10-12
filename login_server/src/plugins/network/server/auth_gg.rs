use super::LoginServerPacketCode;
use l2r_core::{
    model::session::SessionId,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

#[derive(Clone, Copy, Debug)]
pub struct AuthGGResponse {
    session_id: SessionId,
}

impl AuthGGResponse {
    pub fn new(session_id: SessionId) -> Self {
        Self { session_id }
    }
}
impl L2rServerPacket for AuthGGResponse {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::AUTH_GG_RESPONSE.to_le_bytes());
        buffer.u32_from_usize(*self.session_id);
        buffer.extend([0u8; 20]);
        buffer
    }
}
