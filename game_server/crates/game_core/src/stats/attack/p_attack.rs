use crate::stats::*;
use derive_more::{From, Into};
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Into,
)]
pub struct PAtk(u32);
impl PAtk {
    pub fn new(base: BaseClass) -> Self {
        match base {
            BaseClass::Mystic => Self(3),
            _ => Self(10),
        }
    }
    pub fn formula(args: FormulaArguments) -> f32 {
        let str_bonus = args.primal.typed::<STR>(&PrimalStat::STR).bonus();
        args.base_value * str_bonus * StatFormulaRegistry::level_modifier(args.level)
    }
}

impl From<f32> for PAtk {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<PAtk> for f32 {
    fn from(speed: PAtk) -> Self {
        speed.0 as f32
    }
}

impl Default for PAtk {
    fn default() -> Self {
        Self::new(BaseClass::default())
    }
}
