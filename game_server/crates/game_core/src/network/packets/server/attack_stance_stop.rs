use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct AttackStanceStop(ObjectId);
impl fmt::Debug for AttackStanceStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?}> delete", self.0)
    }
}

impl L2rServerPacket for AttackStanceStop {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::ATTACK_STANCE_STOP.to_le_bytes());
        buffer.u32(self.0.into());
        buffer
    }
}

impl AttackStanceStop {
    pub fn new(object_id: ObjectId) -> Self {
        Self(object_id)
    }
}
