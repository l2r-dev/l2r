use crate::stats::*;
use l2r_core::model::base_class::BaseClass;
use strum::EnumIter;

pub struct OtherStatsComponentsPlugin;
impl Plugin for OtherStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OtherStat>()
            .register_type::<OtherStats>()
            .register_type::<UpdateAbnormalSlots>();
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
    fn has_max_pair(&self) -> Option<Self> {
        match self {
            OtherStat::Breath => Some(OtherStat::BreathMax),
            _ => None,
        }
    }

    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        let value = match self {
            OtherStat::FishingExpertise => 0.0,
            OtherStat::Breath => 0.0,
            OtherStat::BreathMax => 0.0,
            OtherStat::MaxBuffSlots => 20.0,
            OtherStat::MaxDebuffSlots => 14.0,
            OtherStat::MaxRhythmSlots => 14.0,
        };
        V::from(value).unwrap_or_default()
    }
}
