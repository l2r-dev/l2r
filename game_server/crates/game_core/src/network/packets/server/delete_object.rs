use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct DeleteObject(ObjectId);
impl fmt::Debug for DeleteObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?}> delete", self.0)
    }
}

impl L2rServerPacket for DeleteObject {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new_16();
        buffer.extend(GameServerPacketCodes::DELETE_OBJECT.to_le_bytes());
        buffer.u32(self.0.into());
        buffer.u32(0);
        buffer
    }
}

impl DeleteObject {
    pub fn new(object_id: ObjectId) -> Self {
        Self(object_id)
    }
}
