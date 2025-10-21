use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct ShowMap;
impl L2rServerPacket for ShowMap {
    fn buffer(self) -> ServerPacketBuffer {
        GameServerPacketCodes::SHOW_MAP
            .to_le_bytes()
            .as_slice()
            .into()
    }
}
