use crate::stats::{FloatStats, StatTrait, StatValue, Stats};
use bevy::{platform::collections::HashMap, prelude::*};
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Component, Debug, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct ProgressRatesStats(FloatStats<ProgressRatesStat>);

impl ProgressRatesStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exp_modifier(&self) -> f32 {
        self.get(&ProgressRatesStat::ExpModifier)
    }

    pub fn sp_modifier(&self) -> f32 {
        self.get(&ProgressRatesStat::SpModifier)
    }
}

impl Default for ProgressRatesStats {
    fn default() -> Self {
        let mut stats = HashMap::default();
        for stat in ProgressRatesStat::iter() {
            stats.insert(stat, stat.default_value::<f32>(BaseClass::default()));
        }
        Self(stats.into())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
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
