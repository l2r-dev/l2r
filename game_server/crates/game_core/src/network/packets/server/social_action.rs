use super::GameServerPacketCodes;
use crate::{action::model::CoreAction, object_id::ObjectId};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::IntoPrimitive;

#[derive(Clone, Debug, Reflect)]
pub struct SocialAction {
    object_id: ObjectId,
    action: Social,
}

impl L2rServerPacket for SocialAction {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::SOCIAL_ACTION.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.action.into());
        buffer
    }
}

impl SocialAction {
    pub fn new(object_id: ObjectId, action: Social) -> Self {
        Self { object_id, action }
    }
}

#[derive(Clone, Copy, Debug, IntoPrimitive, Reflect)]
#[repr(u32)]
pub enum Social {
    Greeting = 2,
    Victory = 3,
    Advance = 4,
    No = 5,
    Yes = 6,
    Bow = 7,
    Unaware = 8,
    Waiting = 9,
    Laugh = 10,
    Applaud = 11,
    Dance = 12,
    Sorrow = 13,
    Charm = 14,
    Shyness = 15,
    LevelUp = 2122,
}

impl TryFrom<CoreAction> for Social {
    type Error = ();

    fn try_from(action: CoreAction) -> Result<Self, Self::Error> {
        match action {
            CoreAction::Greeting => Ok(Social::Greeting),
            CoreAction::Victory => Ok(Social::Victory),
            CoreAction::Advance => Ok(Social::Advance),
            CoreAction::No => Ok(Social::No),
            CoreAction::Yes => Ok(Social::Yes),
            CoreAction::Bow => Ok(Social::Bow),
            CoreAction::Unaware => Ok(Social::Unaware),
            CoreAction::SocialWaiting => Ok(Social::Waiting),
            CoreAction::Laugh => Ok(Social::Laugh),
            CoreAction::Applaud => Ok(Social::Applaud),
            CoreAction::Dance => Ok(Social::Dance),
            CoreAction::Sorrow => Ok(Social::Sorrow),
            CoreAction::Charm => Ok(Social::Charm),
            CoreAction::Shyness => Ok(Social::Shyness),
            _ => Err(()),
        }
    }
}
