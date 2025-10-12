use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use spatial::GameVec3;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct ValidatePosition {
    pub location: Vec3,
}

impl TryFrom<ClientPacketBuffer> for ValidatePosition {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let location = GameVec3::try_from(&mut buffer)?.into();

        Ok(Self { location })
    }
}
