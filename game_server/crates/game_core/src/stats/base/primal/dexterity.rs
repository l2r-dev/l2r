use super::PrimalBonusTable;
use crate::stats::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BONUS_TABLE: LazyLock<[f32; 101]> = LazyLock::new(|| {
    let mut table = [0.0; 101];
    for i in 0..=100u32 {
        table[i as usize] = DEXBonusTable::calculate(i);
    }
    table
});

struct DEXBonusTable;
impl PrimalBonusTable for DEXBonusTable {
    fn calculate(value: u32) -> f32 {
        let exponent = value as f32 - 19.360;
        let bonus = 1.009f32.powf(exponent);
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
pub struct DEX(u32);
impl PrimalStatTrait for DEX {
    fn bonus(&self) -> f32 {
        DEXBonusTable::bonus(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::PrimalStatTrait;

    #[test]
    fn test_bonus() {
        let test_cases = [(1, 0.85), (19, 1.00), (50, 1.32), (99, 2.04)];

        for (value, expected_bonus) in test_cases {
            let dex_stat = super::DEX(value);
            let actual_bonus = dex_stat.bonus();
            assert_eq!(
                actual_bonus, expected_bonus,
                "Failed for DEX({}): expected {}, got {}",
                value, expected_bonus, actual_bonus
            );
        }
    }
}
