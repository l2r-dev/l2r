use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Into,
)]
pub struct MaxBuffSlots(u8);

impl MaxBuffSlots {
    pub const BASIC: u8 = 20;
    pub const MAX: u8 = 25;

    pub fn new(value: u8) -> Self {
        Self(value.min(Self::MAX))
    }
}

impl From<f32> for MaxBuffSlots {
    fn from(value: f32) -> Self {
        Self((value as u8).min(Self::MAX))
    }
}

impl From<MaxBuffSlots> for f32 {
    fn from(slots: MaxBuffSlots) -> Self {
        slots.0 as f32
    }
}

impl Default for MaxBuffSlots {
    fn default() -> Self {
        Self(Self::BASIC)
    }
}
