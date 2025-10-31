use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use spatial::GameVec3;
use std::{convert::TryFrom, fmt::Debug};

#[derive(Deref, Event)]
pub struct TargetNotFound(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Action {
    pub object_id: ObjectId,
    pub origin_location: Vec3,
    pub shift_pressed: bool,
}

impl TryFrom<ClientPacketBuffer> for Action {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let object_id = ObjectId::from(buffer.u32()?);
        let x = buffer.i32()?;
        let y = buffer.i32()?;
        let z = buffer.i32()?;
        let shift_pressed = buffer.bool()?;

        Ok(Self {
            object_id,
            origin_location: GameVec3::new(x, y, z).into(),
            shift_pressed,
        })
    }
}
