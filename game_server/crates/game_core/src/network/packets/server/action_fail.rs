use super::GameServerPacketCodes;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Default, Reflect)]
pub struct ActionFail;
impl L2rServerPacket for ActionFail {
    fn buffer(self) -> ServerPacketBuffer {
        GameServerPacketCodes::ACTION_FAIL
            .to_le_bytes()
            .as_slice()
            .into()
    }
}
