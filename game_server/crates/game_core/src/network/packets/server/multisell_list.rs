use super::GameServerPacketCodes;
use crate::{
    multisell::{Entry, Good, Id as MultisellId},
    network::packets::server::{GameServerPacket, GameServerPackets},
};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct MultisellList {
    id: MultisellId,
    page: u32,
    last_page: bool,
    entries: Vec<Entry>,
}

impl MultisellList {
    pub const PAGE_SIZE: usize = 40;
    pub const MAX_ENTRIES: usize = 800;

    pub fn new(id: MultisellId, mut entries: Vec<Entry>) -> Self {
        entries.truncate(Self::PAGE_SIZE);
        Self {
            id,
            page: 0,
            last_page: true,
            entries,
        }
    }

    pub fn multipage_list(id: MultisellId, mut all_entries: Vec<Entry>) -> GameServerPackets {
        all_entries.truncate(Self::MAX_ENTRIES);
        let total_pages = all_entries.len().div_ceil(Self::PAGE_SIZE);
        let mut packets = Vec::with_capacity(total_pages);

        for page in 0..total_pages {
            let start_index = page * Self::PAGE_SIZE;
            let end_index = std::cmp::min(start_index + Self::PAGE_SIZE, all_entries.len());
            let entries = all_entries
                .get(start_index..end_index)
                .unwrap_or(&[])
                .to_vec();
            let last_page = end_index >= all_entries.len();

            let multisell_list = Self {
                id,
                page: page as u32,
                last_page,
                entries,
            };
            packets.push(GameServerPacket::from(multisell_list));
        }
        packets.into()
    }
}

impl L2rServerPacket for MultisellList {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::MULTI_SELL_LIST.to_le_bytes());
        buffer.u32(self.id.into());
        buffer.u32(self.page + 1);
        buffer.u32(if self.last_page { 1 } else { 0 });
        buffer.u32(Self::PAGE_SIZE as u32);
        let size = std::cmp::min(self.entries.len(), Self::PAGE_SIZE);
        buffer.u32(size as u32);
        for (idx, entry) in self.entries.iter().take(Self::PAGE_SIZE).enumerate() {
            buffer.u32_from_usize(self.page as usize * Self::PAGE_SIZE + idx);
            buffer.u8(if entry.stackable { 1 } else { 0 });
            buffer.u16(0);
            buffer.u32(0);
            buffer.u32(0);
            buffer.u16(65534);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(0);
            buffer.u16(entry.rewards.len() as u16);
            buffer.u16(entry.requirements.len() as u16);
            for reward in &entry.rewards {
                Self::write_good(&mut buffer, reward);
            }
            for requirement in &entry.requirements {
                Self::write_good(&mut buffer, requirement);
            }
        }
        buffer
    }
}

impl MultisellList {
    fn write_good(buffer: &mut ServerPacketBuffer, good: &Good) {
        buffer.u32(good.item.id().into());
        buffer.u32(good.item.bodypart().map_or(0, |bp| bp.into()));
        buffer.u16(good.item.sorting_kind().into());
        buffer.u64(good.item.count());
        buffer.u16(good.item.enchant_level());
        buffer.u32(good.item.augumentation_id().into());
        buffer.i32(good.item.mana().unwrap_or_default());
        buffer.extend(good.item.elements().to_le_bytes());
    }
}
