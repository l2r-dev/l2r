use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Reflect)]
pub struct Die {
    pub object_id: ObjectId,
    pub to_village: u32,
    pub to_hideaway: u32,
    pub to_castle: u32,
    pub to_siege_hq: u32,
    pub sweepable: u32,
    pub self_ressurection: u32,
    pub to_fortress: u32,
}

impl L2rServerPacket for Die {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::DIE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.to_village);
        buffer.u32(self.to_hideaway);
        buffer.u32(self.to_castle);
        buffer.u32(self.to_siege_hq);
        buffer.u32(self.sweepable);
        buffer.u32(self.self_ressurection);
        buffer.u32(self.to_fortress);
        buffer
    }
}
impl Die {
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            to_village: 0,
            to_hideaway: 0,
            to_castle: 0,
            to_siege_hq: 0,
            sweepable: 0,
            self_ressurection: 0,
            to_fortress: 0,
        }
    }
    pub fn to_village(mut self) -> Self {
        self.to_village = 1;
        self
    }
}
