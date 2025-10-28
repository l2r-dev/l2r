use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct SelectTarget {
    pub object_id: ObjectId,
    pub color: u16, // dont know yet pot possibly means that target can be auto-attacked?
}
impl fmt::Debug for SelectTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?}> level_diff: {}", self.object_id, self.color)
    }
}

impl L2rServerPacket for SelectTarget {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::SELECT_TARGET.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u16(self.color);
        buffer.u32(0);
        buffer
    }
}
impl SelectTarget {
    pub fn new(object_id: ObjectId, color: u16) -> Self {
        Self { object_id, color }
    }
}
