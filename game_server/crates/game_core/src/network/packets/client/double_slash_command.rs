use crate::{npc, object_id::ObjectId};
use bevy::prelude::*;
use derive_more::{From, Into};
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, str::FromStr};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, Eq, Hash, PartialEq, Reflect)]
pub enum DoubleSlashCommand {
    #[default]
    Unknown,
    #[strum(serialize = "admin")]
    Admin,
    #[strum(serialize = "spawn")]
    Spawn(npc::Id),
    #[strum(serialize = "goto")]
    GoTo { target_obj_id: ObjectId },
}

impl FromStr for DoubleSlashCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();

        let command = match parts.next().map(|s| s.to_lowercase()) {
            Some(cmd) => cmd,
            _ => return Err(()),
        };

        match command.as_str() {
            "admin" => Ok(DoubleSlashCommand::Admin),
            "spawn" => {
                let id = match parts.next() {
                    Some(id) => id.parse::<usize>().unwrap_or_default(),
                    _ => return Err(()),
                };
                Ok(DoubleSlashCommand::Spawn(npc::Id::from(id)))
            }
            "goto" => {
                let Some(object_id) = parts.next() else {
                    return Err(());
                };

                let Ok(object_id) = object_id.parse::<usize>() else {
                    return Err(());
                };

                Ok(DoubleSlashCommand::GoTo {
                    target_obj_id: ObjectId::from(object_id),
                })
            }
            _ => Err(()),
        }
    }
}

impl TryFrom<ClientPacketBuffer> for DoubleSlashCommand {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let command = buffer.str()?;
        Ok(Self::from_str(&command).unwrap_or_default())
    }
}

#[derive(Debug, Event, From, Into)]
pub struct DoubleSlashCommandExecuted(pub DoubleSlashCommand);
