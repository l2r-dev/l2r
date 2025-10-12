use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, From, Into, PartialEq, Reflect, Serialize)]
pub struct MDef(u32);

impl MDef {
    pub fn formula(args: FormulaArguments) -> f32 {
        let men_bonus = args.primal.typed::<MEN>(&PrimalStat::MEN).bonus();
        let base_value = if args.is_pet {
            args.base_value - 13.0
        } else {
            args.base_value
        };
        base_value * men_bonus * StatFormulaRegistry::level_modifier(args.level)
    }
}

impl From<MDef> for f32 {
    fn from(value: MDef) -> Self {
        value.0 as f32
    }
}
