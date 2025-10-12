use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deref,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    From,
    Into,
    Hash,
)]
pub struct PAtkRange(u32);
impl PAtkRange {
    pub const BASE: u32 = 40;
}

impl From<f32> for PAtkRange {
    fn from(value: f32) -> Self {
        Self(value as u32)
    }
}

impl From<PAtkRange> for f32 {
    fn from(value: PAtkRange) -> Self {
        value.0 as f32
    }
}
