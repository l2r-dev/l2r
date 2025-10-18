use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter};

mod class;
mod movement;
mod primal;
mod vitals;

pub use class::*;
pub use movement::*;
pub use primal::*;
pub use vitals::*;

#[repr(usize)]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumCount,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
)]
pub enum MpConsumptionStat {
    PhysicalMpConsumeRate,
    MagicalMpConsumeRate,
    DanceMpConsumeRate,
    BowMpConsumeRate,
    MpConsume,
}
