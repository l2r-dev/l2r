use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt};

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct CharSlot(pub u32);
impl fmt::Display for CharSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CharSlot({})", self.0)
    }
}

#[derive(Clone, Default, PartialEq, Reflect)]
pub struct CharacterSelect {
    pub char_slot: CharSlot,
}

impl fmt::Debug for CharacterSelect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "char_slot: {:?}", self.char_slot.0)
    }
}

impl TryFrom<ClientPacketBuffer> for CharacterSelect {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let char_slot = buffer.u32()?;

        Ok(Self {
            char_slot: CharSlot(char_slot),
        })
    }
}
