use crate::chat::UserCommand;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Reflect)]
pub struct SingleSlashCommand {
    pub command: UserCommand,
}

impl fmt::Debug for SingleSlashCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserCommand: {:?}", self.command)
    }
}

impl TryFrom<ClientPacketBuffer> for SingleSlashCommand {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let command = if buffer.remaining() < 4 {
            UserCommand::Location
        } else {
            let cmd_bytes = buffer.bytes(4)?;
            UserCommand::from(cmd_bytes)
        };

        Ok(Self { command })
    }
}
