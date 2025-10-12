use crate::{object_id::ObjectId, skills};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestDispel {
    pub object_id: ObjectId,
    pub skill_id: skills::Id,
    pub skill_level: skills::Level,
}

impl TryFrom<ClientPacketBuffer> for RequestDispel {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let object_id = ObjectId::from(buffer.u32()?);
        let skill_id = skills::Id::from(buffer.u32()?);
        let skill_level = skills::Level::from(buffer.u32()?);

        Ok(Self {
            object_id,
            skill_id,
            skill_level,
        })
    }
}
