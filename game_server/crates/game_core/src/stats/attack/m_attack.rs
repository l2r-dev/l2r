use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deserialize,
    Deref,
    DerefMut,
    PartialEq,
    Reflect,
    Serialize,
    From,
    Into,
)]
pub struct MAtk(u32);
impl MAtk {
    pub fn formula(args: FormulaArguments) -> f32 {
        let int_bonus = args.primal.typed::<INT>(&PrimalStat::INT).bonus();
        let lvl_bonus = StatFormulaRegistry::level_modifier(args.level);
        args.base_value * int_bonus.powi(2) * lvl_bonus.powi(2)
    }
}

impl From<f32> for MAtk {
    fn from(value: f32) -> Self {
        Self(value as u32)
    }
}

impl From<MAtk> for f32 {
    fn from(value: MAtk) -> Self {
        value.0 as f32
    }
}
