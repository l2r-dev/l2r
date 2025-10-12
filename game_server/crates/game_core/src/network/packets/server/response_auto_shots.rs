use super::GameServerPacketCodes;
use crate::items::Id;
use bevy::prelude::*;
use derive_more::From;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Clone, Debug, Reflect)]
pub struct ResponseAutoShots {
    item_id: Id,
    state: ShotState,
}

#[derive(Component, Debug, Deref, From, Reflect)]
pub struct AutoShotUse(Id);

#[derive(Clone, Copy, Debug, Default, FromPrimitive, IntoPrimitive, PartialEq, Reflect)]
#[repr(u32)]
pub enum ShotState {
    #[default]
    Off,
    On,
}

impl L2rServerPacket for ResponseAutoShots {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::EX_AUTO_SHOTS.to_le_bytes());
        buffer.u32(self.item_id.into());
        buffer.u32(self.state.into());
        buffer
    }
}
impl ResponseAutoShots {
    pub fn new(item_id: Id, state: ShotState) -> Self {
        ResponseAutoShots { item_id, state }
    }
}
