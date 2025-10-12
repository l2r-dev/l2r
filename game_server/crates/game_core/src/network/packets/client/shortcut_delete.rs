use crate::shortcut::SlotId;
use bevy::prelude::*;
use derive_more::{From, Into};
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt::Debug};

#[derive(Clone, Debug, From, Into, PartialEq, Reflect)]
pub struct RequestShortcutDelete(pub SlotId);

impl TryFrom<ClientPacketBuffer> for RequestShortcutDelete {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        Ok(Self(SlotId::from(buffer.u32()?)))
    }
}
