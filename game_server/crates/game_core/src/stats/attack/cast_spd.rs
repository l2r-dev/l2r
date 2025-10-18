use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Deserialize,
    Deref,
    DerefMut,
    PartialEq,
    Reflect,
    Serialize,
    From,
    Into,
)]
pub struct CastSpd(u32);

impl CastSpd {
    pub const BASE: u32 = 333;
    pub const MAX: u32 = 2000;
    pub fn formula(args: FormulaArguments) -> f32 {
        let wit_bonus = args.primal.typed::<WIT>(PrimalStat::WIT).bonus();
        args.base_value * wit_bonus
    }
}

impl Default for CastSpd {
    fn default() -> Self {
        Self(Self::BASE)
    }
}

impl From<f32> for CastSpd {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<CastSpd> for f32 {
    fn from(speed: CastSpd) -> Self {
        speed.0 as f32
    }
}
