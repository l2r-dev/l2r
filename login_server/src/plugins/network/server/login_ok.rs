use super::LoginServerPacketCode;
use core::fmt;
use l2r_core::{
    crypt::session_keys::SessionKey,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

#[derive(Clone)]
pub struct LoginOk {
    session_key: SessionKey,
}
impl fmt::Debug for LoginOk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("").finish()
    }
}
impl L2rServerPacket for LoginOk {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(LoginServerPacketCode::LOGIN_OK.to_le_bytes());
        buffer.extend(self.session_key.get_login().to_le_bytes());
        buffer.u32(0);
        buffer.u32(0);
        buffer.u32(1002);
        for _ in 0..6 {
            buffer.u32(0);
        }
        buffer
    }
}
impl LoginOk {
    pub fn new(session_key: SessionKey) -> Self {
        Self { session_key }
    }
}
