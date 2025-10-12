use super::GameServerPacketCodes;
use bevy::prelude::*;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Reflect)]
pub struct EtcStatusUpdate;
impl fmt::Debug for EtcStatusUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EtcStatusUpdate")
    }
}
impl L2rServerPacket for EtcStatusUpdate {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::ETC_STATUS_UPDATE.to_le_bytes());
        buffer.u32(0); // 1-7 increase force (force charges), level
        buffer.u32(0); // 1-4 weight penalty, level (1=50%, 2=66.6%, 3=80%, 4=100%)
        buffer.u32(0); // 1 = block all chat
        buffer.u32(0); // 1 = danger area
        buffer.u32(0); // Weapon Grade Penalty [1-4]
        buffer.u32(0); // Armor Grade Penalty [1-4]
        buffer.u32(0); // 1 = charm of courage (allows resurrection on the same spot upon death on the siege battlefield)
        buffer.u32(0); // 1-15 death penalty, level (combat ability decreased due to death)
        buffer
    }
}
impl Default for EtcStatusUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl EtcStatusUpdate {
    pub fn new() -> Self {
        Self
    }
}
