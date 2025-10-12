use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Reflect)]
pub struct UseItem {
    pub object_id: ObjectId,
    pub ctrl_pressed: bool,
}

impl fmt::Debug for UseItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "UseItem {{ object_id: {:?}, ctrl_pressed: {:?} }}",
            self.object_id, self.ctrl_pressed
        )
    }
}

impl TryFrom<ClientPacketBuffer> for UseItem {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let object_id = ObjectId::from(buffer.u32()?);
        let ctrl_pressed = buffer.u32()? == 1;
        Ok(Self {
            object_id,
            ctrl_pressed,
        })
    }
}
