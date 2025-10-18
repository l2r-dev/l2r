use bevy::prelude::*;
use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Hash,
)]
pub struct PAtkWidth(u32);
impl PAtkWidth {
    pub const BASE: u32 = 90;
    pub const MAX: u32 = 360;
}

impl From<f32> for PAtkWidth {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<PAtkWidth> for f32 {
    fn from(speed: PAtkWidth) -> Self {
        speed.0 as f32
    }
}

impl Default for PAtkWidth {
    fn default() -> Self {
        Self(Self::BASE)
    }
}
