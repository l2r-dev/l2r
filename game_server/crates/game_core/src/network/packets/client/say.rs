use crate::chat::{self};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Reflect)]
pub struct Say {
    pub text: String,
    pub chat_type: chat::Kind,
    pub target: Option<String>,
}

impl fmt::Debug for Say {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]: {} - {}",
            self.chat_type,
            self.text,
            self.target.as_deref().unwrap_or("")
        )
    }
}

impl TryFrom<ClientPacketBuffer> for Say {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let text = buffer.str()?;
        let chat_type = if buffer.remaining() < 4 {
            chat::Kind::General
        } else {
            chat::Kind::from(buffer.bytes(4)?)
        };
        let target = if chat_type == chat::Kind::Whisper && buffer.remaining() > 0 {
            Some(buffer.str()?)
        } else {
            None
        };
        Ok(Self {
            text,
            chat_type,
            target,
        })
    }
}
