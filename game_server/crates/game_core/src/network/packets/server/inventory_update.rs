use super::GameServerPacketCodes;
use crate::items::*;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use smallvec::SmallVec;

#[derive(Clone, Debug, Reflect)]
pub struct InventoryUpdate {
    pub items: SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]>,
    pub update_type: UpdateType,
}

impl L2rServerPacket for InventoryUpdate {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::INVENTORY_UPDATE.to_le_bytes());
        buffer.u16_from_usize(self.items.len());
        for unique in self.items {
            buffer.u16(self.update_type.into());
            buffer.extend(unique.to_le_bytes());
        }
        buffer
    }
}

impl InventoryUpdate {
    pub fn new(
        items: SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]>,
        update_type: UpdateType,
    ) -> Self {
        Self { items, update_type }
    }
}
