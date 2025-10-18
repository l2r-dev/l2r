use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize)]
pub struct MagicCriticalRate(u32);
impl MagicCriticalRate {
    pub const MAX: f32 = 500.0;
    pub fn formula(args: FormulaArguments) -> f32 {
        let wit_bonus = args.primal.typed::<WIT>(PrimalStat::WIT).bonus();
        args.base_value * wit_bonus * 10.0
    }
}

impl From<f32> for MagicCriticalRate {
    fn from(value: f32) -> Self {
        Self(value as u32)
    }
}

impl From<MagicCriticalRate> for f32 {
    fn from(value: MagicCriticalRate) -> Self {
        value.0 as f32
    }
}
