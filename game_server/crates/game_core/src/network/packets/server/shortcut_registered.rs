use super::GameServerPacketCodes;
use crate::shortcut::Shortcut;
use bevy::prelude::*;
use derive_more::derive::From;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, From, Reflect)]
pub struct ShortcutRegistered(Shortcut);

impl L2rServerPacket for ShortcutRegistered {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::SHORT_CUT_REGISTER.to_le_bytes());
        buffer.extend(self.0.into_buffer().to_vec());
        buffer
    }
}
