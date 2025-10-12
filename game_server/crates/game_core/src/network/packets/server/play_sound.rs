use super::GameServerPacketCodes;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct PlaySound {
    _unknown1: u32,
    sound_file: String,
    _unknown3: u32,
    _unknown4: u32,
    _unknown5: u32,
    _unknown6: u32,
    _unknown7: u32,
    _unknown8: u32,
}

impl L2rServerPacket for PlaySound {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::PLAY_SOUND.to_le_bytes());
        buffer.u32(self._unknown1);
        buffer.str(self.sound_file.as_str());
        buffer.u32(self._unknown3);
        buffer.u32(self._unknown4);
        buffer.u32(self._unknown5);
        buffer.u32(self._unknown6);
        buffer.u32(self._unknown7);
        buffer.u32(self._unknown8);
        buffer
    }
}
impl PlaySound {
    pub fn new(sound_file: String) -> Self {
        PlaySound {
            _unknown1: 0,
            sound_file,
            _unknown3: 0,
            _unknown4: 0,
            _unknown5: 0,
            _unknown6: 0,
            _unknown7: 0,
            _unknown8: 0,
        }
    }
}
