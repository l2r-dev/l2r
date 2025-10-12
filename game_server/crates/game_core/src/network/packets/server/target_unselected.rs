use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::fmt;

#[derive(Clone, Reflect)]
pub struct TargetUnselected {
    object_id: ObjectId,
    location: Vec3,
}
impl fmt::Debug for TargetUnselected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?}> loc: {}", self.object_id, self.location)
    }
}

impl L2rServerPacket for TargetUnselected {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        let location = GameVec3::from(self.location);

        buffer.extend(GameServerPacketCodes::TARGET_UNSELECTED.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.extend(location.to_le_bytes());
        buffer.u32(0);
        buffer
    }
}

impl TargetUnselected {
    pub fn new(object_id: ObjectId, location: Vec3) -> Self {
        Self {
            object_id,
            location,
        }
    }
}
