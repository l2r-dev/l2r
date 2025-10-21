use super::GameServerPacketCodes;
use bevy::prelude::Reflect;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Default, Reflect)]
pub struct CharacterCreationSuccess;
impl fmt::Debug for CharacterCreationSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
impl L2rServerPacket for CharacterCreationSuccess {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new_6();
        buffer.extend(GameServerPacketCodes::CHARACTER_CREATE_SUCCESS.to_le_bytes());
        buffer.u32(1);
        buffer
    }
}
