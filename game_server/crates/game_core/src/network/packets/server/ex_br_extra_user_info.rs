use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct ExBrExtraUserInfo {
    object_id: ObjectId,
    effect: u32,
    mark: u8,
}
impl L2rServerPacket for ExBrExtraUserInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::EX_BR_EXTRA_USER_INFO.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.effect);
        buffer.u8(self.mark);
        buffer
    }
}
impl ExBrExtraUserInfo {
    pub fn new(object_id: ObjectId, effect: u32, mark: u8) -> Self {
        Self {
            object_id,
            effect,
            mark,
        }
    }
}
