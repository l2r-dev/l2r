use super::GameServerPacketCodes;
use crate::{object_id::ObjectId, stats::MovementStat};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct ChangeMoveType {
    object_id: ObjectId,
    move_type: MovementStat,
}

impl L2rServerPacket for ChangeMoveType {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::CHANGE_MOVE_TYPE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32_from_usize(self.move_type.into());
        buffer.u8(0x00);
        buffer
    }
}

impl ChangeMoveType {
    pub fn new(object_id: ObjectId, move_type: MovementStat) -> Self {
        Self {
            object_id,
            move_type,
        }
    }
}
