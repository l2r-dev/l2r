use crate::{object_id::ObjectId, teleport::TeleportListKind};
use bevy::reflect::Reflect;
use std::str::FromStr;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumDiscriminants, Eq, Hash, PartialEq, Reflect,
)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(ChatCommandVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ChatCommand {
    Tp(TeleportListKind),
    Number(u32),
}

impl FromStr for ChatCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "index" => Ok(ChatCommand::Number(0)), // "index" command is treated as page 0
            // Tp commands start with "tp" followed by the kind (e.g., "tp", "tp_noble", "tp_floor")
            tp if tp.starts_with("tp") => {
                if tp == "tp" {
                    // Handle plain "tp" command (use default)
                    Ok(ChatCommand::Tp(TeleportListKind::default()))
                } else if let Some(kind_str) = tp.strip_prefix("tp_") {
                    // Handle "tp_<kind>" format
                    // Skip "tp_"
                    match TeleportListKind::from_str(kind_str) {
                        Ok(kind) => Ok(ChatCommand::Tp(kind)),
                        Err(_) => Err(format!("Unknown teleport kind: {kind_str}")),
                    }
                } else {
                    Err(format!("Invalid tp command format: {s}"))
                }
            }

            _ => {
                // Try to parse as a page number
                if let Ok(page) = s.parse::<u32>() {
                    Ok(ChatCommand::Number(page))
                } else {
                    Err(format!("Invalid chat command: {s}"))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct NpcAction {
    pub npc_oid: ObjectId,
    pub command: NpcCommand,
}

impl FromStr for NpcAction {
    type Err = String;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        // All npc commands start with "npc_", it cutted on top level
        // when building BypassCommand, now need to parse npc id and rest of the command
        let mut parts = command.splitn(2, '_');
        let npc_id = parts.next().unwrap_or("");
        let command = parts.next().unwrap_or("");

        let npc_oid = npc_id
            .parse::<ObjectId>()
            .map_err(|_| format!("Invalid NPC OID: {npc_id}"))?;

        let command = NpcCommand::from_str(command)?;

        Ok(Self { npc_oid, command })
    }
}

#[derive(Clone, Debug, Display, EnumDiscriminants, Eq, Hash, PartialEq, Reflect)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(NpcCommandVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum NpcCommand {
    Tp(crate::teleport::Id),
    Chat(ChatCommand),
    Quest(String),
    Multisell(u32),
}

impl FromStr for NpcCommand {
    type Err = String;
    fn from_str(command: &str) -> Result<Self, String> {
        let mut parts = command.splitn(2, ' ');
        let base_command = parts.next().unwrap_or("");
        let arg = parts.next();

        let variant = NpcCommandVariants::from_str(base_command)
            .map_err(|_| format!("Unknown NPC command: {base_command}"))?;

        match variant {
            NpcCommandVariants::Tp => {
                if let Some(arg) = arg {
                    // Space-separated the item id and count
                    let mut item_parts = arg.splitn(2, ' ');
                    if let Some(item_id_str) = item_parts.next() {
                        let item_id = item_id_str
                            .parse::<crate::teleport::Id>()
                            .map_err(|_| format!("Invalid TP ID: {item_id_str}"))?;

                        return Ok(NpcCommand::Tp(item_id));
                    }
                }

                Err(format!(
                    "Invalid or missing argument for spawn_item: {command}"
                ))
            }

            NpcCommandVariants::Chat => {
                if let Some(arg) = arg {
                    return Ok(NpcCommand::Chat(ChatCommand::from_str(arg)?));
                }

                Err(format!(
                    "Invalid or missing argument for chat command: {command}"
                ))
            }

            NpcCommandVariants::Quest => {
                if let Some(arg) = arg {
                    return Ok(NpcCommand::Quest(arg.to_string()));
                }

                Err(format!(
                    "Invalid or missing argument for quest command: {command}"
                ))
            }

            NpcCommandVariants::Multisell => {
                if let Some(arg) = arg {
                    let item_id = arg
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid multisell ID: {arg}"))?;
                    return Ok(NpcCommand::Multisell(item_id));
                }

                Err(format!(
                    "Invalid or missing argument for multisell command: {command}"
                ))
            }
        }
    }
}
