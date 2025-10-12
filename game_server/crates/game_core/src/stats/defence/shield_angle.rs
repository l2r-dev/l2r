use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct ShieldAngle(u32);

impl ShieldAngle {
    pub const BASE: u32 = 120;
    pub fn formula(args: FormulaArguments) -> f32 {
        args.base_value
    }
}
