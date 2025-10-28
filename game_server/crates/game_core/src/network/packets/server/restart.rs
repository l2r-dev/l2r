use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct Restart(bool);

impl Restart {
    pub fn new(allowed: bool) -> Self {
        Self(allowed)
    }
}

impl L2rServerPacket for Restart {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::RESTART_RESPONSE.to_le_bytes());
        buffer.u32_from_bool(self.0);
        buffer
    }
}
