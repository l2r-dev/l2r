use super::GameServerPacketCodes;
use crate::{items::Id, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;

#[derive(Clone, Debug, Reflect)]
pub struct GetItem {
    pickuper: ObjectId,
    item: ObjectId,
    item_id: Id,
    location: Vec3,
}

impl L2rServerPacket for GetItem {
    fn buffer(self) -> ServerPacketBuffer {
        let location = GameVec3::from(self.location);

        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::GET_ITEM.to_le_bytes());
        buffer.u32(self.pickuper.into());
        buffer.u32(self.item.into());
        buffer.extend(location.to_le_bytes());
        buffer
    }
}

impl GetItem {
    pub fn new(pickuper: ObjectId, item: ObjectId, item_id: Id, location: Vec3) -> Self {
        Self {
            pickuper,
            item,
            item_id,
            location,
        }
    }
}
