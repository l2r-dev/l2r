use crate::stats::{FloatStats, StatTrait, StatValue, Stats};
use bevy::prelude::*;
use l2r_core::model::base_class::BaseClass;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter};

#[derive(
    Clone, Component, Debug, Default, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize,
)]
#[serde(default)]
pub struct ProgressRatesStats(FloatStats<ProgressRatesStat>);

impl ProgressRatesStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exp_modifier(&self) -> f32 {
        self.get(ProgressRatesStat::ExpModifier)
    }

    pub fn sp_modifier(&self) -> f32 {
        self.get(ProgressRatesStat::SpModifier)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    TryFromPrimitive,
    IntoPrimitive,
    EnumCount,
)]
#[repr(usize)]
pub enum ProgressRatesStat {
    ExpModifier,
    SpModifier,
    BonusExp,
    BonusSp,
    VitalityConsumeRate,
    MaxSouls,
    ExpLostByPvp,
    ExpLostByMob,
    ExpLostByRaid,
    DeathPenaltyByPvp,
    DeathPenaltyByMob,
    DeathPenaltyByRaid,
    TryFromPrimitive,
    IntoPrimitive,
    EnumCount,
}

impl StatTrait for ProgressRatesStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        use ProgressRatesStat::*;
        let value = match self {
            ExpModifier => 1.0,
            SpModifier => 1.0,
            ExpLostByPvp => 1.0,
            ExpLostByMob => 1.0,
            ExpLostByRaid => 1.0,
            _ => 0.0,
        };
        V::from(value).unwrap_or_default()
    }

    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::from(1000.0).unwrap_or_default()
    }
}
