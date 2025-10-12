use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize)]
pub struct CriticalRate(u32);
impl CriticalRate {
    pub fn formula(args: FormulaArguments) -> f32 {
        let dex_bouns = args.primal.typed::<DEX>(&PrimalStat::DEX).bonus();
        args.base_value * dex_bouns * 10.0
    }
}

impl From<f32> for CriticalRate {
    fn from(value: f32) -> Self {
        Self(value as u32)
    }
}

impl From<CriticalRate> for f32 {
    fn from(value: CriticalRate) -> Self {
        value.0 as f32
    }
}
