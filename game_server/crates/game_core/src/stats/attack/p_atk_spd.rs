use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(
    Clone, Component, Copy, Debug, Deref, Deserialize, PartialEq, Reflect, Serialize, From, Into,
)]
pub struct PAtkSpd(u32);
impl PAtkSpd {
    pub const BASE: u32 = 300;
    pub fn get_attack_speed_multiplier(&self) -> f64 {
        (1.1 * self.0 as f64) / Self::BASE as f64
    }

    pub fn set(&mut self, speed: u32) {
        self.0 = speed;
    }

    pub fn attack_interval(&self) -> Duration {
        let delay_ms = 500000 / if self.0 == 0 { 1 } else { self.0 };
        Duration::from_millis(delay_ms as u64)
    }

    pub fn formula(args: FormulaArguments) -> f32 {
        let dex_bonus = args.primal.typed::<DEX>(&super::PrimalStat::DEX).bonus();
        args.base_value * dex_bonus
    }
}

impl From<f32> for PAtkSpd {
    fn from(speed: f32) -> Self {
        Self(speed as u32)
    }
}

impl From<PAtkSpd> for f32 {
    fn from(speed: PAtkSpd) -> Self {
        speed.0 as f32
    }
}

impl Default for PAtkSpd {
    fn default() -> Self {
        Self(Self::BASE)
    }
}
