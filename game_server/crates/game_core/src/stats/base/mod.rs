use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

mod class;
mod movement;
mod primal;
mod vitals;

pub use class::*;
pub use movement::*;
pub use primal::*;
pub use vitals::*;

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
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
