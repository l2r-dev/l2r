use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, IntoPrimitive, PartialEq, Reflect, TryFromPrimitive)]
pub enum CharacterDeletionFailReason {
    #[default]
    DeletionFailed = 1,
    YouMayNotDeleteClanMember = 2,
    ClanLeadersMayNotBeDeleted = 3,
}

#[derive(Clone, Default, Reflect)]
pub struct CharacterDeletionFailed(CharacterDeletionFailReason);

impl fmt::Debug for CharacterDeletionFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl CharacterDeletionFailed {
    pub fn new(reason: CharacterDeletionFailReason) -> Self {
        Self(reason)
    }
}

impl L2rServerPacket for CharacterDeletionFailed {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::CHARACTER_DELETE_FAIL.to_le_bytes());
        buffer.u32(self.0.into());
        buffer
    }
}
