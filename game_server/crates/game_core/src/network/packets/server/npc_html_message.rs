use super::GameServerPacketCodes;
use crate::{items::Id, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use strum::Display;

#[derive(Clone, Copy, Debug, Display, Reflect)]
pub enum HtmlKind {
    Npc,
    NpcItem,
    NpcQuest,
    Tutorial,
    CommunityBoard,
}

#[derive(Clone, Debug, Reflect)]
pub struct NpcHtmlMessage {
    object_id: ObjectId,
    item_id: Id,
    html: String,
}

impl NpcHtmlMessage {
    // Maximum packet size is 65535 bytes
    // Packet overhead: 1 byte (packet code) + 4 bytes (object_id) + 4 bytes (item_id) = 9 bytes
    // String overhead: 2 bytes (null terminator)
    // Available for string content: 65535 - 9 - 2 = 65524 bytes
    // Each UTF-16 character takes 2 bytes, so max characters: 65524 / 2 = 32762
    const MAX_HTML_CHARS: usize = 10000;
    pub fn new(object_id: ObjectId, mut html: String, item_id: Id) -> Self {
        html.truncate(Self::MAX_HTML_CHARS);

        Self {
            object_id,
            item_id,
            html,
        }
    }
}

impl L2rServerPacket for NpcHtmlMessage {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::NPC_HTML_MESSAGE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.str(&self.html);
        buffer.u32(self.item_id.into());
        buffer
    }
}
