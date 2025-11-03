use crate::{items, npc, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, str::FromStr};
use strum::EnumIter;
use util_macros::EnumFromArgsDeserialize;

/// In-game usage: `//commandname arg1 arg2 ...`
/// For example, for `GoTo` input will be `//goto 1234`
#[derive(Clone, Debug, Default, EnumFromArgsDeserialize, EnumIter, PartialEq, Reflect)]
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
    Immortal,

    //Ingame GmPanel Commands teleportto char_name / instant_move / summon item_id count / summon 1_000_000 + npc_id count
    TeleportTo {
        target_name: String,
    },
    InstantMove,
    Summon {
        id: u32,
        count: u64,
    },

    ///Ctrl+Shift Left Click on game map
    Teleport {
        x: f32,
        z: f32,
    },
    Open,
    Close,
}

impl TryFrom<ClientPacketBuffer> for DoubleSlashCommand {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let command = buffer.str()?;
        Ok(Self::from_str(&command).unwrap_or_default())
    }
}
