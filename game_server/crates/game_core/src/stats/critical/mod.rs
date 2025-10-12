use bevy::prelude::*;
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

mod critical_rate;
mod magic_critical_rate;

use crate::stats::*;
pub use critical_rate::*;
pub use magic_critical_rate::*;
use spatial::RelativeDirection;

pub struct CriticalComponentsPlugin;
impl Plugin for CriticalComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CriticalStats>()
            .register_type::<CriticalStat>();

        app.world_mut()
            .resource_mut::<StatFormulaRegistry>()
            .register_formula(CriticalStat::CriticalRate.into(), CriticalRate::formula)
            .register_formula(
                CriticalStat::MagicCriticalRate.into(),
                MagicCriticalRate::formula,
            );
    }
}

#[derive(
    Clone, Component, Debug, Default, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize,
)]
#[serde(default)]
pub struct CriticalStats(FloatStats<CriticalStat>);

impl CriticalStats {
    pub fn positional_damage(&self, direction: RelativeDirection) -> f32 {
        match direction {
            RelativeDirection::Back => self.get(&CriticalStat::CriticalDamageBack),
            RelativeDirection::Side => self.get(&CriticalStat::CriticalDamageSide),
            RelativeDirection::Face => self.get(&CriticalStat::CriticalDamageFront),
        }
    }
}

impl AsRef<GenericStats<CriticalStat, f32>> for CriticalStats {
    fn as_ref(&self) -> &GenericStats<CriticalStat, f32> {
        &self.0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum CriticalStat {
    CriticalDamage,
    CriticalDamageFront,
    CriticalDamageBack,
    CriticalDamageSide,
    CriticalDamageAdditional,
    MagicCriticalDamage,
    CriticalRate,
    CriticalRateFront,
    CriticalRateBack,
    CriticalRateSide,
    BlowRate,
    MagicCriticalRate,
    AttackCancel,
}

impl StatTrait for CriticalStat {
    fn default_value<V: StatValue>(&self, base_class: BaseClass) -> V {
        let value = match self {
            CriticalStat::CriticalDamage => 1.0,
            CriticalStat::CriticalDamageFront => 1.0,
            CriticalStat::CriticalDamageBack => 1.0,
            CriticalStat::CriticalDamageSide => 1.0,
            CriticalStat::CriticalDamageAdditional => 0.0,
            CriticalStat::MagicCriticalDamage => 1.0,
            CriticalStat::CriticalRate => base_class.base_critical_rate(),
            CriticalStat::CriticalRateFront => 1.0,
            CriticalStat::CriticalRateBack => 1.0,
            CriticalStat::CriticalRateSide => 1.0,
            CriticalStat::BlowRate => 10.0,
            CriticalStat::MagicCriticalRate => 1.0,
            CriticalStat::AttackCancel => 1.0,
        };
        V::from(value).unwrap_or_default()
    }
}
