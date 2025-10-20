use crate::{items, npc, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, str::FromStr};
use strum::EnumIter;
use util_macros::EnumFromArgsDeserialize;

/// In-game usage: `//commandname arg1 arg2 ...`
/// For example, for `GoTo` input will be `//goto 1234`
#[derive(
    Clone, Copy, Debug, Default, EnumIter, Eq, Hash, PartialEq, Reflect, EnumFromArgsDeserialize,
)]
pub enum DoubleSlashCommand {
    #[default]
    Unknown,
    Admin,
    Spawn {
        npc_id: npc::Id,
    },
    GoTo {
        target_obj_id: ObjectId,
    },
    Item {
        id: items::Id,
        count: u64,
    },
}

impl TryFrom<ClientPacketBuffer> for DoubleSlashCommand {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let command = buffer.str()?;
        Ok(Self::from_str(&command).unwrap_or_default())
    }
}
