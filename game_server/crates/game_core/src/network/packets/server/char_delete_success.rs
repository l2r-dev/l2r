use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Default, Reflect)]
pub struct CharacterDeletionSuccess;
impl fmt::Debug for CharacterDeletionSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl L2rServerPacket for CharacterDeletionSuccess {
    fn buffer(self) -> ServerPacketBuffer {
        GameServerPacketCodes::CHARACTER_DELETE_SUCCESS
            .to_le_bytes()
            .as_slice()
            .into()
    }
}
