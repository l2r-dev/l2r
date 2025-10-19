use super::GameServerPacketCodes;
use crate::{items::Id, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::fmt;

#[derive(Clone, Reflect)]
pub struct DropItem {
    dropper: ObjectId,
    item_oid: ObjectId,
    item_id: Id,
    loc: Vec3,
    is_stackable: bool,
    count: u64,
}

impl fmt::Debug for DropItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:?}> DropItem [item: {}, display_id: {}, position: ({}), stackable: {}, count: {}, dropper: {}]",
            GameServerPacketCodes::DROP_ITEM,
            self.item_oid,
            self.item_id,
            self.loc,
            self.is_stackable,
            self.count,
            self.dropper,
        )
    }
}

impl L2rServerPacket for DropItem {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        let loc = GameVec3::from(self.loc);
        buffer.extend(GameServerPacketCodes::DROP_ITEM.to_le_bytes());
        buffer.u32(self.dropper.into());
        buffer.u32(self.item_oid.into());
        buffer.u32(self.item_id.into());
        buffer.extend(loc.to_le_bytes());
        buffer.u32_from_bool(self.is_stackable);
        buffer.u64(self.count);
        buffer.u32(1);
        buffer
    }
}

impl DropItem {
    pub fn new(
        dropper: ObjectId,
        item_oid: ObjectId,
        item_id: Id,
        loc: Vec3,
        is_stackable: bool,
        count: u64,
    ) -> Self {
        Self {
            dropper,
            item_oid,
            item_id,
            loc,
            is_stackable,
            count,
        }
    }
}
