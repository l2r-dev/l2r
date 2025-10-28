use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::{GameVec3, Heading};

#[derive(Clone, Debug, Reflect)]
pub struct StopMove {
    object_id: ObjectId,
    transform: Transform,
}

impl L2rServerPacket for StopMove {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();

        let location = GameVec3::from(self.transform.translation);
        let heading = Heading::from(self.transform.rotation);

        buffer.extend(GameServerPacketCodes::STOP_MOVE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.extend(location.to_le_bytes());
        buffer.i32(heading.into());
        buffer
    }
}

impl StopMove {
    pub fn new(object_id: ObjectId, transform: Transform) -> Self {
        Self {
            object_id,
            transform,
        }
    }
}
