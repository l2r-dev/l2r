use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use spatial::GameVec3;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestDropItem {
    pub object_id: ObjectId,
    pub count: u64,
    pub location: Vec3,
}

impl TryFrom<ClientPacketBuffer> for RequestDropItem {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let object_id = ObjectId::from(buffer.u32()?);
        let count = buffer.u64()?;
        let location = GameVec3::try_from(&mut buffer)?.into();

        Ok(Self {
            object_id,
            count,
            location,
        })
    }
}
