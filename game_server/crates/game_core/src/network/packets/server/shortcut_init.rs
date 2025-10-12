use super::GameServerPacketCodes;
use bevy::prelude::*;
use derive_more::derive::From;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, From, Reflect)]
pub struct ShortcutInit(Vec<crate::shortcut::Shortcut>);

impl L2rServerPacket for ShortcutInit {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::SHORT_CUT_INIT.to_le_bytes());
        buffer.u32_from_usize(self.0.len());
        for shortcut in self.0 {
            buffer.extend(shortcut.into_buffer().to_vec());
        }
        buffer
    }
}
