use crate::stats::*;
use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From,
)]
pub struct Evasion(u32);
impl Evasion {
    pub fn formula(args: FormulaArguments) -> f32 {
        let dex = args.primal.get(&PrimalStat::DEX) as f32;
        let level: f32 = args.level.into();

        let mut value = args.base_value + (dex.sqrt() * 6.0) + level;

        if args.is_character {
            if level >= 70.0 {
                let mut diff = level - 69.0;

                if level >= 78.0 {
                    diff *= 1.2;
                }

                value += diff;
            }
        } else if level > 69.0 {
            value += (level - 69.0) + 2.0;
        }

        value
    }
}

impl From<f32> for Evasion {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<Evasion> for f32 {
    fn from(speed: Evasion) -> Self {
        speed.0 as f32
    }
}

impl Default for Evasion {
    fn default() -> Self {
        Self(5)
    }
}
