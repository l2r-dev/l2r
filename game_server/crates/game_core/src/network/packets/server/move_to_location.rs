use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::fmt;

#[derive(Clone, Reflect)]
pub struct MoveToLocation {
    object_id: ObjectId,
    origin_location: Vec3,
    target_location: Vec3,
}

impl fmt::Debug for MoveToLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> current: {}, target: {}",
            self.object_id, self.origin_location, self.target_location
        )
    }
}

impl L2rServerPacket for MoveToLocation {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        let origin_location = GameVec3::from(self.origin_location);
        let target_location = GameVec3::from(self.target_location);
        buffer.extend(GameServerPacketCodes::MOVE_TO_LOCATION.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.extend(target_location.to_le_bytes());
        buffer.extend(origin_location.to_le_bytes());
        buffer
    }
}

impl MoveToLocation {
    pub fn new(object_id: ObjectId, origin_location: Vec3, target_location: Vec3) -> Self {
        Self {
            object_id,
            origin_location,
            target_location,
        }
    }
}
