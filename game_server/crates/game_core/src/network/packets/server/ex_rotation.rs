use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::Heading;

#[derive(Clone, Debug, Reflect)]
pub struct ExRotation {
    object_id: ObjectId,
    heading: Heading,
}
impl L2rServerPacket for ExRotation {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::EX_ROTATION.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.i32(self.heading.into());
        buffer
    }
}
impl ExRotation {
    pub fn new(object_id: ObjectId, heading: Heading) -> Self {
        Self { object_id, heading }
    }
}
