use super::PrimalBonusTable;
use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BONUS_TABLE: LazyLock<[f32; 101]> = LazyLock::new(|| {
    let mut table = [0.0; 101];
    for i in 0..=100u32 {
        table[i as usize] = CONBonusTable::calculate(i);
    }
    table
});

struct CONBonusTable;
impl PrimalBonusTable for CONBonusTable {
    fn calculate(value: u32) -> f32 {
        let exponent = value as f32 - 27.632;
        let bonus = 1.030f32.powf(exponent);
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
pub struct CON(u32);
impl PrimalStatTrait for CON {
    fn bonus(&self) -> f32 {
        CONBonusTable::bonus(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::PrimalStatTrait;

    #[test]
    fn test_bonus() {
        let test_cases = [(1, 0.46), (50, 1.94), (99, 8.24)];

        for (value, expected_bonus) in test_cases {
            let con_stat = super::CON(value);
            let actual_bonus = con_stat.bonus();
            assert_eq!(
                actual_bonus, expected_bonus,
                "Failed for CON({}): expected {}, got {}",
                value, expected_bonus, actual_bonus
            );
        }
    }
}
