use crate::skills::{self};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestMagicSkillUse {
    pub skill_id: skills::Id,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
}

impl TryFrom<ClientPacketBuffer> for RequestMagicSkillUse {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let skill_id = skills::Id::from(buffer.u32()?);
        let ctrl_pressed = buffer.bool_from_u32()?;
        let shift_pressed = buffer.bool()?;

        Ok(Self {
            skill_id,
            ctrl_pressed,
            shift_pressed,
        })
    }
}
