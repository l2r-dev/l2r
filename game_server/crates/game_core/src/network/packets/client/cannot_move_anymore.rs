use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use spatial::GameVec3;
use std::convert::TryFrom;

/// Sent by client when player stops moving (reached destination or collision)
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct CannotMoveAnymore {
    pub location: Vec3,
    pub heading: i32,
}

impl TryFrom<ClientPacketBuffer> for CannotMoveAnymore {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let location = GameVec3::try_from(&mut buffer)?.into();
        let heading = buffer.i32()?;

        Ok(Self { location, heading })
    }
}
