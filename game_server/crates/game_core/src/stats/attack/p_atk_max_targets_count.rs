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
pub struct PAtkMaxTargetsCount(u32);
impl PAtkMaxTargetsCount {
    pub const BASE: u32 = 1;
}

impl From<f32> for PAtkMaxTargetsCount {
    fn from(value: f32) -> Self {
        Self(value as u32)
    }
}

impl From<PAtkMaxTargetsCount> for f32 {
    fn from(value: PAtkMaxTargetsCount) -> Self {
        value.0 as f32
    }
}
