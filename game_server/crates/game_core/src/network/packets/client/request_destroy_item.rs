use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestDestroyItem {
    pub object_id: ObjectId,
    pub count: u64,
}

impl TryFrom<ClientPacketBuffer> for RequestDestroyItem {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let object_id = ObjectId::from(buffer.u32()?);
        let count = buffer.u64()?;

        Ok(Self { object_id, count })
    }
}
