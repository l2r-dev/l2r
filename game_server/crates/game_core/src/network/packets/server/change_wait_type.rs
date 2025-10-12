use super::GameServerPacketCodes;
use crate::{action::wait_kind::WaitKind, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;

#[derive(Clone, Debug, Reflect)]
pub struct ChangeWaitType {
    object_id: ObjectId,
    wait_kind: WaitKind,
    location: Vec3,
}

impl L2rServerPacket for ChangeWaitType {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        let location = GameVec3::from(self.location);

        buffer.extend(GameServerPacketCodes::CHANGE_WAIT_TYPE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.wait_kind.into());
        buffer.extend(location.to_le_bytes());
        buffer
    }
}

impl ChangeWaitType {
    pub fn new(object_id: ObjectId, wait_kind: WaitKind, location: Vec3) -> Self {
        Self {
            object_id,
            wait_kind,
            location,
        }
    }
}
