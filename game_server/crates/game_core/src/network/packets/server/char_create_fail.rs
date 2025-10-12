use super::GameServerPacketCodes;
use crate::character;
use bevy::prelude::*;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, IntoPrimitive, PartialEq, Reflect, TryFromPrimitive)]
pub enum CharacterCreationFailReason {
    #[default]
    CreationFailed,
    TooManyCharacters,
    NameAlreadyExists,
    SixteenEngChars,
    IncorrectName,
    CreateNotAllowed,
    ChooseAnotherSvr,
}

impl From<character::TableError> for CharacterCreationFailReason {
    fn from(err: character::TableError) -> Self {
        match err {
            character::TableError::MaxCharsReached => Self::TooManyCharacters,
            character::TableError::InvalidCharSlot => Self::CreationFailed,
        }
    }
}

#[derive(Clone, Default, Reflect)]
pub struct CharacterCreationFailed(CharacterCreationFailReason);
impl fmt::Debug for CharacterCreationFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
impl L2rServerPacket for CharacterCreationFailed {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::CHARACTER_CREATE_FAIL.to_le_bytes());
        buffer.u32(self.0.into());
        buffer
    }
}
impl CharacterCreationFailed {
    pub fn new(reason: CharacterCreationFailReason) -> Self {
        Self(reason)
    }
}
