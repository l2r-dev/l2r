use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Default, Reflect)]
pub struct LogoutOk;

impl L2rServerPacket for LogoutOk {
    fn buffer(self) -> ServerPacketBuffer {
        GameServerPacketCodes::LOG_OUT_OK
            .to_le_bytes()
            .as_slice()
            .into()
    }
}
