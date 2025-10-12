use super::PrimalBonusTable;
use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BONUS_TABLE: LazyLock<[f32; 101]> = LazyLock::new(|| {
    let mut table = [0.0; 101];
    for i in 0..=100u32 {
        table[i as usize] = WITBonusTable::calculate(i);
    }
    table
});

struct WITBonusTable;
impl PrimalBonusTable for WITBonusTable {
    fn calculate(value: u32) -> f32 {
        let exponent = value as f32 - 20.000;
        let bonus = 1.050f32.powf(exponent);
        (bonus * 100.0).round() / 100.0
    }

    fn table() -> &'static LazyLock<[f32; 101]> {
        &BONUS_TABLE
    }
}

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deref,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    From,
    Into,
)]
pub struct WIT(u32);
impl PrimalStatTrait for WIT {
    fn bonus(&self) -> f32 {
        WITBonusTable::bonus(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::PrimalStatTrait;

    #[test]
    fn test_bonus() {
        let test_cases = [
            (1, 0.40),
            (10, 0.61),
            (20, 1.00),
            (30, 1.63),
            (50, 4.32),
            (75, 14.64),
            (99, 47.20),
        ];

        for (value, expected_bonus) in test_cases {
            let wit_stat = super::WIT(value);
            let actual_bonus = wit_stat.bonus();
            assert_eq!(
                actual_bonus, expected_bonus,
                "Failed for WIT({}): expected {}, got {}",
                value, expected_bonus, actual_bonus
            );
        }
    }
}
