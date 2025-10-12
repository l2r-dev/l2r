use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use system_messages::{Id, SmParam};

#[derive(Clone, Reflect)]
pub struct SystemMessage {
    // This stored as u32 to easily use it from scripts, to not convert it to Id every time
    id: u32,
    message_params: Vec<SmParam>,
}
impl fmt::Debug for SystemMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} - {:?}", self.id, self.message_params)
    }
}
impl L2rServerPacket for SystemMessage {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        let param_count = self.message_params.len() as u32;
        buffer.extend(GameServerPacketCodes::SYSTEM_MESSAGE.to_le_bytes());
        buffer.u32(self.id);
        buffer.u32(param_count);
        for param in self.message_params {
            buffer.extend(param.to_le_bytes());
        }
        buffer
    }
}
impl SystemMessage {
    pub fn new(id: Id, message_params: Vec<SmParam>) -> Self {
        Self {
            id: id.u32(),
            message_params,
        }
    }

    pub fn new_empty(id: Id) -> Self {
        Self {
            id: id.u32(),
            message_params: Vec::new(),
        }
    }

    pub fn new_with_int_id(id: u32, message_params: Vec<SmParam>) -> Self {
        Self { id, message_params }
    }
}
