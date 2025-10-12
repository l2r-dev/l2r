use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, Default, IntoPrimitive, Reflect)]
#[repr(u16)]
pub enum SevenSigns {
    DAWN = 1,
    #[default]
    DUSK = 2,
}

#[derive(Clone, Default, Reflect)]
pub struct SSQInfo(SevenSigns);
impl fmt::Debug for SSQInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "winning side: {:?}", self.0)
    }
}
impl L2rServerPacket for SSQInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::SSQ_INFO.to_le_bytes());
        buffer.u16(self.0.into());
        buffer
    }
}
impl SSQInfo {
    pub fn new(ssq_winner: SevenSigns) -> Self {
        SSQInfo(ssq_winner)
    }
}
