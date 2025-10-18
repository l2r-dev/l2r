use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct ShieldRate(u32);

impl ShieldRate {
    pub const BASE: f32 = 10.0;
    pub const MAX: f32 = 90.0;
    pub fn formula(args: FormulaArguments) -> f32 {
        let dex_bonus = args.primal.typed::<DEX>(super::PrimalStat::DEX).bonus();
        args.base_value * dex_bonus
    }
}
