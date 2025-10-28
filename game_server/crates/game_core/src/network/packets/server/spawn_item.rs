use super::GameServerPacketCodes;
use crate::{items, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;

#[derive(Clone, Debug, Reflect)]
pub struct SpawnItem {
    object_id: ObjectId,
    item_id: items::Id,
    location: Vec3,
    stackable: bool,
    count: u64,
}

impl L2rServerPacket for SpawnItem {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        let location = GameVec3::from(self.location);
        buffer.extend(GameServerPacketCodes::SPAWN_ITEM.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.item_id.into());
        buffer.extend(location.to_le_bytes());
        buffer.u32_from_bool(self.stackable);
        buffer.u64(self.count);
        buffer.u32(0);
        buffer.u32(0);
        buffer
    }
}

impl SpawnItem {
    pub fn new(
        object_id: ObjectId,
        item_id: items::Id,
        location: Vec3,
        stackable: bool,
        count: u64,
    ) -> Self {
        Self {
            object_id,
            item_id,
            location,
            stackable,
            count,
        }
    }
}
