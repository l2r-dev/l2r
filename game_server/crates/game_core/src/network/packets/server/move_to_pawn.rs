use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::fmt;

#[derive(Clone, Reflect)]
pub struct MoveToPawn {
    origin_location: Vec3,
    target_location: Vec3,
    moving: ObjectId,
    target: ObjectId,
    distance: u32,
}
impl fmt::Debug for MoveToPawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}->{}> current: {}, target: {}, distance: {},",
            self.moving, self.target, self.origin_location, self.target_location, self.distance
        )
    }
}
impl L2rServerPacket for MoveToPawn {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();

        let origin_location = GameVec3::from(self.origin_location);
        let target_location = GameVec3::from(self.target_location);

        buffer.extend(GameServerPacketCodes::MOVE_TO_PAWN.to_le_bytes());
        buffer.u32(self.moving.into());
        buffer.u32(self.target.into());
        buffer.u32(self.distance);
        buffer.extend(origin_location.to_le_bytes());
        buffer.extend(target_location.to_le_bytes());
        buffer
    }
}
impl MoveToPawn {
    pub fn new(
        moving: ObjectId,
        target: ObjectId,
        origin_location: Vec3,
        target_location: Vec3,
        distance: u32,
    ) -> Self {
        Self {
            origin_location,
            target_location,
            moving,
            target,
            distance,
        }
    }
}
