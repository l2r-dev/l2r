use crate::stats::*;
use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From,
)]
pub struct Accuracy(u32);
impl Accuracy {
    pub const MAX: u32 = 500;
    pub fn formula(args: FormulaArguments) -> f32 {
        let dex = args.primal.get(PrimalStat::DEX) as f32;
        let level: f32 = args.level.into();

        if level > 77.0 {
            args.base_value + (dex.sqrt() * 6.0) + level + (level - 76.0)
        } else if level > 69.0 {
            args.base_value + (dex.sqrt() * 6.0) + level + (level - 69.0)
        } else {
            args.base_value + (dex.sqrt() * 6.0) + level
        }
    }
}

impl From<f32> for Accuracy {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<Accuracy> for f32 {
    fn from(speed: Accuracy) -> Self {
        speed.0 as f32
    }
}

impl Default for Accuracy {
    fn default() -> Self {
        Self(5)
    }
}
