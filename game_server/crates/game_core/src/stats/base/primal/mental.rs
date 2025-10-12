use super::PrimalBonusTable;
use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BONUS_TABLE: LazyLock<[f32; 101]> = LazyLock::new(|| {
    let mut table = [0.0; 101];
    for i in 0..=100u32 {
        table[i as usize] = MENBonusTable::calculate(i);
    }
    table
});

struct MENBonusTable;
impl PrimalBonusTable for MENBonusTable {
    fn calculate(value: u32) -> f32 {
        let exponent = value as f32 + 0.060;
        let bonus = 1.010f32.powf(exponent);
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
pub struct MEN(u32);
impl PrimalStatTrait for MEN {
    fn bonus(&self) -> f32 {
        MENBonusTable::bonus(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::PrimalStatTrait;

    #[test]
    fn test_bonus() {
        let test_cases = [
            (1, 1.01),
            (10, 1.11),
            (20, 1.22),
            (30, 1.35),
            (50, 1.65),
            (75, 2.11),
            (99, 2.68),
        ];

        for (value, expected_bonus) in test_cases {
            let men_stat = super::MEN(value);
            let actual_bonus = men_stat.bonus();
            assert_eq!(
                actual_bonus, expected_bonus,
                "Failed for MEN({}): expected {}, got {}",
                value, expected_bonus, actual_bonus
            );
        }
    }
}
