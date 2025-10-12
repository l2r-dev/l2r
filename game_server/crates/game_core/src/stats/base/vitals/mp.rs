use crate::stats::*;
use derive_more::{From, Into};
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Into,
)]
pub struct Mp(u32);
impl Mp {
    pub fn new(base: BaseClass) -> Self {
        match base {
            BaseClass::Mystic => Self(3),
            _ => Self(10),
        }
    }

    pub fn formula(args: FormulaArguments) -> f32 {
        if !args.is_character || args.is_pet {
            return args.base_value;
        }

        let con_bonus = args.primal.typed::<CON>(&PrimalStat::CON).bonus();
        args.base_value * con_bonus
    }
}

impl From<f32> for Mp {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<Mp> for f32 {
    fn from(speed: Mp) -> Self {
        speed.0 as f32
    }
}

impl Default for Mp {
    fn default() -> Self {
        Self::new(BaseClass::default())
    }
}
