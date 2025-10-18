use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Into,
)]
pub struct MaxRhythmSlots(u8);

impl MaxRhythmSlots {
    pub const BASIC: u8 = 14;
    pub const MAX: u8 = 14;

    pub fn new(value: u8) -> Self {
        Self(value.min(Self::MAX))
    }
}

impl From<f32> for MaxRhythmSlots {
    fn from(value: f32) -> Self {
        Self((value as u8).min(Self::MAX))
    }
}

impl From<MaxRhythmSlots> for f32 {
    fn from(slots: MaxRhythmSlots) -> Self {
        slots.0 as f32
    }
}

impl Default for MaxRhythmSlots {
    fn default() -> Self {
        Self(Self::BASIC)
    }
}
