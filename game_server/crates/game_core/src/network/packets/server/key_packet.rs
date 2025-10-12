use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::{
    crypt::blowfish::BlowfishKey,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

#[derive(Clone, Default, Reflect)]
pub struct KeyPacket {
    blowfish_key: BlowfishKey,
}
impl fmt::Debug for KeyPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyPacket")
    }
}
impl L2rServerPacket for KeyPacket {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::KEY_PACKET.to_le_bytes());
        buffer.u8(1); // 0 - wrong, 1 - protocol ok
        buffer.extend(self.blowfish_key.to_le_bytes()[0..8].to_vec());
        buffer.u32(1); // use blowfish encryption
        buffer.u32(0); // server id
        buffer.u8(0);
        buffer.u32(0); // obfuscation key
        buffer
    }
}
impl KeyPacket {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn build(mut self, blowfish_key: BlowfishKey) -> Self {
        self.blowfish_key = blowfish_key;
        self
    }
}
