use crate::{items::Id, network::packets::server::ShotState};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::convert::TryFrom;

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct RequestAutoShots {
    pub item_id: Id,
    pub state: ShotState,
}

impl TryFrom<ClientPacketBuffer> for RequestAutoShots {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let item_id = buffer.u32()?.into();
        let state = buffer.u32()?.into();
        Ok(Self { item_id, state })
    }
}
