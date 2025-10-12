use super::PrimalBonusTable;
use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BONUS_TABLE: LazyLock<[f32; 101]> = LazyLock::new(|| {
    let mut table = [0.0; 101];
    for i in 0..=100u32 {
        table[i as usize] = INTBonusTable::calculate(i);
    }
    table
});

struct INTBonusTable;
impl PrimalBonusTable for INTBonusTable {
    fn calculate(value: u32) -> f32 {
        let exponent = value as f32 - 31.375;
        let bonus = 1.020f32.powf(exponent);
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
pub struct INT(u32);
impl PrimalStatTrait for INT {
    fn bonus(&self) -> f32 {
        INTBonusTable::bonus(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::PrimalStatTrait;

    #[test]
    fn test_bonus() {
        let test_cases = [(1, 0.55), (50, 1.45), (99, 3.82)];

        for (value, expected_bonus) in test_cases {
            let int_stat = super::INT(value);
            let actual_bonus = int_stat.bonus();
            assert_eq!(
                actual_bonus, expected_bonus,
                "Failed for INT({}): expected {}, got {}",
                value, expected_bonus, actual_bonus
            );
        }
    }
}
