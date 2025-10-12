use crate::movement::MoveMode;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use num_enum::TryFromPrimitive;
use spatial::GameVec3;
use std::{
    convert::TryFrom,
    fmt::{self, Debug},
};

#[derive(Clone, PartialEq, Reflect)]
pub struct MoveBackwardToLocation {
    pub target_location: Vec3,
    pub origin_location: Vec3,
    pub move_mode: MoveMode,
}

impl Debug for MoveBackwardToLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "target_location: {}, origin_location: {}, move_mode: {}",
            self.target_location, self.origin_location, self.move_mode
        )
    }
}

impl TryFrom<ClientPacketBuffer> for MoveBackwardToLocation {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let target_location = GameVec3::try_from(&mut buffer)?.into();
        let origin_location = GameVec3::try_from(&mut buffer)?.into();

        // TODO: check it, if it's correct
        let move_mode = if buffer.remaining() >= 4 {
            MoveMode::try_from_primitive(buffer.u32()?)?
        } else {
            MoveMode::Keyboard
        };

        Ok(Self {
            target_location,
            origin_location,
            move_mode,
        })
    }
}
