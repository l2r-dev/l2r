use super::LoginServerPacketCode;
use l2r_core::{
    crypt::session_keys::SessionKey,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

#[derive(Clone)]
pub struct PlayOk {
    session_key: SessionKey,
}

impl std::fmt::Debug for PlayOk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{:?}> PlayOk", LoginServerPacketCode::PLAY_OK)
    }
}

impl L2rServerPacket for PlayOk {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::PLAY_OK.to_le_bytes());
        buffer.i64(self.session_key.get_game());
        buffer
    }
}
impl PlayOk {
    pub fn new(key: SessionKey) -> Self {
        Self { session_key: key }
    }
}
