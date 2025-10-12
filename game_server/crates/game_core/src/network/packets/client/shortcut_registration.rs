use crate::shortcut::{ShortcutKind, ShortcutKindVariant, ShortcutTargetKind, SlotId};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, fmt::Debug};

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct RequestShortcutRegistration {
    pub kind: ShortcutKind,
    pub slot_id: SlotId,
    pub level: u32,
    pub target: ShortcutTargetKind,
}

impl TryFrom<ClientPacketBuffer> for RequestShortcutRegistration {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let shortcut_variant =
            ShortcutKindVariant::try_from_primitive(buffer.u32()?).unwrap_or_default();

        let slot_id = SlotId::from(buffer.u32()?);

        let shortcut_id = buffer.u32()?;
        let level = buffer.u32()?;

        let target = ShortcutTargetKind::try_from_primitive(buffer.u32()?).unwrap_or_default();

        let kind = ShortcutKind::new(shortcut_variant, shortcut_id, level.into())
            .map_err(|err| L2rSerializeError::new(err.to_string(), buffer.as_slice()))?;

        Ok(Self {
            kind,
            slot_id,
            level,
            target,
        })
    }
}
