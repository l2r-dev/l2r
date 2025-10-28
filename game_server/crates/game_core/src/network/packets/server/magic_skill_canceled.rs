use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct MagicSkillCanceled(ObjectId);

impl MagicSkillCanceled {
    pub fn new(id: ObjectId) -> Self {
        Self(id)
    }
}

impl L2rServerPacket for MagicSkillCanceled {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::MAGIC_SKILL_CANCELED.to_le_bytes());
        buffer.u32(self.0.into());
        buffer
    }
}
