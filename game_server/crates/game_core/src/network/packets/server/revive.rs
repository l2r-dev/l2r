use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct Revive(ObjectId);

impl L2rServerPacket for Revive {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::REVIVE.to_le_bytes());
        buffer.u32(self.0.into());
        buffer
    }
}
impl Revive {
    pub fn new(object_id: ObjectId) -> Self {
        Self(object_id)
    }
}
