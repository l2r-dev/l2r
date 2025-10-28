use super::GameServerPacketCodes;
use crate::{chat, network::broadcast::BroadcastScope, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct CreatureSay {
    chat_type: chat::Kind,
    object_id: ObjectId,
    char_name: String,
    text: Vec<String>,
    recievers: Option<Vec<Entity>>,
}
impl fmt::Debug for CreatureSay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:?}> [{:?}] {:?}: {:?}",
            self.object_id,
            self.chat_type,
            self.char_name,
            self.text.join("\n")
        )
    }
}
impl L2rServerPacket for CreatureSay {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::CREATURE_SAY.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.chat_type.into());
        buffer.str(&self.char_name);
        // now send -1 for npc string id cause i dont know what it is
        buffer.i32(-1);
        buffer.str(&self.text.join("\n"));
        buffer
    }
}
impl CreatureSay {
    pub fn new(
        object_id: ObjectId,
        char_name: String,
        text: Vec<String>,
        chat_type: chat::Kind,
        recievers: Option<Vec<Entity>>,
    ) -> Self {
        Self {
            chat_type,
            object_id,
            char_name,
            text,
            recievers,
        }
    }
    pub fn broadcast_scope(&self) -> BroadcastScope {
        match self.chat_type {
            chat::Kind::General => BroadcastScope::Radius(1250.0),
            chat::Kind::NpcGeneral => BroadcastScope::Radius(1250.0),
            chat::Kind::Boat => BroadcastScope::Radius(1250.0),
            chat::Kind::Shout => BroadcastScope::InRegion,
            chat::Kind::NpcShout => BroadcastScope::InRegion,
            chat::Kind::Trade => BroadcastScope::InRegion,
            chat::Kind::Whisper => {
                BroadcastScope::Entities(self.recievers.clone().unwrap_or_default())
            }
            chat::Kind::Party => {
                BroadcastScope::Entities(self.recievers.clone().unwrap_or_default())
            }
            chat::Kind::Clan => {
                BroadcastScope::Entities(self.recievers.clone().unwrap_or_default())
            }
            chat::Kind::Gm => BroadcastScope::All,
            chat::Kind::Announcement => BroadcastScope::All,
            chat::Kind::CriticalAnnounce => BroadcastScope::All,
            chat::Kind::ScreenAnnounce => BroadcastScope::All,
            chat::Kind::Hero => BroadcastScope::All,
            _ => BroadcastScope::Known,
        }
    }
}
