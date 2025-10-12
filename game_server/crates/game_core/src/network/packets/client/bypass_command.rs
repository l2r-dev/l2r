use crate::{admin_menu::AdminMenuCommand, npc::NpcAction};
use bevy::{log, prelude::*};
use derive_more::{From, Into};
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, str::FromStr};
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

#[derive(Clone, Debug, Display, EnumDiscriminants, PartialEq, Reflect)]
#[strum_discriminants(name(BypassCommandVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "lowercase"))]
pub enum BypassCommand {
    Unknown(String),
    Admin(AdminMenuCommand),
    Npc(NpcAction),
}

impl FromStr for BypassCommand {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, strum::ParseError> {
        let mut parts = s.splitn(2, '_');
        let category = parts.next().unwrap_or("");
        let command = parts.next().unwrap_or("");

        match BypassCommandVariants::from_str(category) {
            Ok(BypassCommandVariants::Admin) => match AdminMenuCommand::from_str(command) {
                Ok(admin_command) => Ok(BypassCommand::Admin(admin_command)),
                Err(err) => {
                    log::warn!("{}", err);
                    Ok(BypassCommand::Unknown(format!(
                        "Admin command parsing error: {err}"
                    )))
                }
            },

            Ok(BypassCommandVariants::Npc) => match NpcAction::from_str(command) {
                Ok(npc_action) => Ok(BypassCommand::Npc(npc_action)),
                Err(err) => {
                    log::warn!("{}", err);
                    Ok(BypassCommand::Unknown(format!(
                        "NPC command parsing error: {err}"
                    )))
                }
            },
            _ => Ok(BypassCommand::Unknown(s.to_string())),
        }
    }
}

impl TryFrom<ClientPacketBuffer> for BypassCommand {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let command = buffer.str()?;
        let result = Self::from_str(&command);

        result.is_err().then(|| {
            log::warn!("Failed to parse bypass command: {}", command);
        });

        Ok(result.unwrap_or(BypassCommand::Unknown(command)))
    }
}

#[derive(Debug, Event, From, Into)]
pub struct BypassCommandExecuted(pub BypassCommand);
