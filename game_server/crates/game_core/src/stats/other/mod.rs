use crate::stats::*;
use l2r_core::model::base_class::BaseClass;
use strum::EnumIter;

mod max_buff_slots;
mod max_debuff_slots;
mod max_rhythm_slots;

pub use max_buff_slots::*;
pub use max_debuff_slots::*;
pub use max_rhythm_slots::*;

pub struct OtherStatsComponentsPlugin;
impl Plugin for OtherStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OtherStat>()
            .register_type::<OtherStats>()
            .register_type::<UpdateAbnormalSlots>()
            .register_type::<MaxBuffSlots>()
            .register_type::<MaxDebuffSlots>()
            .register_type::<MaxRhythmSlots>();
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, PartialEq, Reflect)]
pub struct OtherStats(FloatStats<OtherStat>);

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct UpdateAbnormalSlots;

impl Default for OtherStats {
    fn default() -> Self {
        let base_class = BaseClass::default();
        let mut float_stats = FloatStats::default();
        for stat in OtherStat::iter() {
            float_stats.insert(stat, stat.default_value(base_class));
        }
        Self(float_stats)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum OtherStat {
    FishingExpertise,
    Breath,
    BreathMax,
    MaxBuffSlots,
    MaxDebuffSlots,
    MaxRhythmSlots,
}

impl OtherStat {
    pub fn buff_slot_changed(&self) -> bool {
        matches!(
            self,
            OtherStat::MaxBuffSlots | OtherStat::MaxDebuffSlots | OtherStat::MaxRhythmSlots
        )
    }
}

impl StatTrait for OtherStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        let value = match self {
            OtherStat::FishingExpertise => 0.0,
            OtherStat::Breath => 0.0,
            OtherStat::BreathMax => 0.0,
            OtherStat::MaxBuffSlots => MaxBuffSlots::BASIC.into(),
            OtherStat::MaxDebuffSlots => MaxDebuffSlots::BASIC.into(),
            OtherStat::MaxRhythmSlots => MaxRhythmSlots::BASIC.into(),
        };
        V::from(value).unwrap_or_default()
    }

    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        let value = match self {
            OtherStat::FishingExpertise => f32::MAX,
            OtherStat::Breath => f32::MAX,
            OtherStat::BreathMax => f32::MAX,
            OtherStat::MaxBuffSlots => MaxBuffSlots::MAX.into(),
            OtherStat::MaxDebuffSlots => MaxDebuffSlots::MAX.into(),
            OtherStat::MaxRhythmSlots => MaxRhythmSlots::MAX.into(),
        };
        V::from(value).unwrap_or_default()
    }

    fn has_max_pair(&self) -> Option<Self> {
        match self {
            OtherStat::Breath => Some(OtherStat::BreathMax),
            _ => None,
        }
    }
}
