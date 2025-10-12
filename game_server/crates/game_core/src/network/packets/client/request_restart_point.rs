use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{convert::TryFrom, fmt};

#[derive(Clone, Copy, Debug, Default, IntoPrimitive, Reflect, TryFromPrimitive)]
#[repr(u32)]
pub enum RequestedPoint {
    #[default]
    Town,
    ClanHall,
    Castle,
    Fortress,
    SiegeHQ,
    FestivalParticipant,
    AgathionRes,
    Jail,
}

#[derive(Clone, Copy, Event)]
pub struct Respawn;

#[derive(Clone, Copy, Reflect)]
pub struct RequestRestartPoint {
    pub point: RequestedPoint,
}

impl fmt::Debug for RequestRestartPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RequestRestartPoint: {:?}", self.point)
    }
}

impl TryFrom<ClientPacketBuffer> for RequestRestartPoint {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let point = RequestedPoint::try_from(buffer.u32()?)?;
        Ok(Self { point })
    }
}
