use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct PDef(u32);

impl PDef {
    pub fn formula(args: FormulaArguments) -> f32 {
        args.base_value * StatFormulaRegistry::level_modifier(args.level)
    }
}
