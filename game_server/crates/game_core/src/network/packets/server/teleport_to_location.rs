use super::GameServerPacketCodes;
use crate::{object_id::ObjectId, teleport::TeleportType};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::{GameVec3, Heading};

#[derive(Clone, Debug, Event, Reflect)]
pub struct TeleportToLocation {
    object_id: ObjectId,
    transform: Transform,
    tp_type: TeleportType,
}

impl TeleportToLocation {
    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub fn tp_type(&self) -> TeleportType {
        self.tp_type
    }
}

impl L2rServerPacket for TeleportToLocation {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();

        let location = GameVec3::from(self.transform.translation);
        let heading = Heading::from(self.transform.rotation);

        buffer.extend(GameServerPacketCodes::TELEPORT_TO_LOCATION.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.extend(location.to_le_bytes());
        buffer.u32(self.tp_type.into());
        buffer.i32(heading.into());
        buffer
    }
}

impl TeleportToLocation {
    pub fn new(object_id: ObjectId, transform: Transform, tp_type: TeleportType) -> Self {
        Self {
            object_id,
            transform,
            tp_type,
        }
    }
}
