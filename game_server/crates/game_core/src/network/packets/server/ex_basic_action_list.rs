use super::GameServerPacketCodes;
use crate::action::model::DEFAULT_ACTIONS;
use bevy::prelude::Reflect;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct ExBasicActionList;

impl L2rServerPacket for ExBasicActionList {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::EX_BASIC_ACTION_LIST.to_le_bytes());
        buffer.u32_from_usize(DEFAULT_ACTIONS.len());
        for action_id in DEFAULT_ACTIONS.into_iter() {
            buffer.u32(action_id);
        }
        buffer
    }
}
