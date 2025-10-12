use crate::action::model::*;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestActionUse {
    pub action_id: ActionId,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
}

impl TryFrom<ClientPacketBuffer> for RequestActionUse {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let action_int = buffer.u32()?;
        let ctrl_pressed = buffer.bool()?;
        let mut shift_pressed = false;

        let action_id = ActionId::try_from(action_int)
            .map_err(|err| L2rSerializeError::new(err.to_string(), buffer.as_slice()))?;

        if buffer.remaining() > 0 {
            shift_pressed = buffer.bool()?;
        }

        Ok(Self {
            action_id,
            ctrl_pressed,
            shift_pressed,
        })
    }
}
