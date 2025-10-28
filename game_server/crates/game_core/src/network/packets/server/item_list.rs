use super::GameServerPacketCodes;
use crate::items::UniqueItem;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct ItemList {
    pub items: Vec<UniqueItem>,
    pub show_window: bool,
}

impl L2rServerPacket for ItemList {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::ITEM_LIST.to_le_bytes());
        buffer.u16_from_bool(self.show_window);
        buffer.u16_from_usize(self.items.len());
        for unique in self.items {
            buffer.extend(unique.to_le_bytes());
        }
        // TODO: inventory blocking
        buffer.u16(0);
        buffer
    }
}
impl ItemList {
    pub fn new(items: Vec<UniqueItem>, show_window: bool) -> Self {
        Self { items, show_window }
    }
}
