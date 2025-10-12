use crate::multisell::Id;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Reflect)]
pub struct MultisellChoose {
    list_id: Id,
    entry_id: u32,
    amount: u64,
}

impl MultisellChoose {
    pub fn list_id(&self) -> Id {
        self.list_id
    }

    pub fn entry_id(&self) -> u32 {
        self.entry_id
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl fmt::Debug for MultisellChoose {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MultisellChoose {{ list_id: {:?}, entry_id: {:?}, amount: {}}}",
            self.list_id, self.entry_id, self.amount,
        )
    }
}

impl TryFrom<ClientPacketBuffer> for MultisellChoose {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let list_id = buffer.u32()?.into();
        let entry_id = buffer.u32()?;
        let amount = buffer.u64()?;
        let _unknown1 = buffer.u16()?;
        let _unknown2 = buffer.u32()?;
        let _unknown3 = buffer.u32()?;
        let _unknown4 = buffer.u16()?;
        let _unknown5 = buffer.u16()?;
        let _unknown6 = buffer.u16()?;
        let _unknown7 = buffer.u16()?;
        let _unknown8 = buffer.u16()?;
        let _unknown9 = buffer.u16()?;
        let _unknown10 = buffer.u16()?;
        let _unknown11 = buffer.u16()?;

        Ok(Self {
            list_id,
            entry_id,
            amount,
        })
    }
}
