use crate::stats::*;
use bevy::platform::collections::HashMap;
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

mod accuracy;
mod cast_spd;
mod effect_kind;
mod m_attack;
mod p_atk_range;
mod p_atk_spd;
mod p_attack;
mod p_attack_width;

pub use accuracy::*;
pub use cast_spd::*;
pub use effect_kind::*;
pub use m_attack::*;
pub use p_atk_range::*;
pub use p_atk_spd::*;
pub use p_attack::*;
pub use p_attack_width::*;

pub struct AttackStatsComponentsPlugin;
impl Plugin for AttackStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackStats>()
            .register_type::<AttackStat>()
            .register_type::<PAtk>()
            .register_type::<PAtkSpd>()
            .register_type::<CastSpd>()
            .register_type::<Accuracy>()
            .register_type::<PAtkRange>();

        app.world_mut()
            .resource_mut::<StatFormulaRegistry>()
            .register_formula(AttackStat::PAtk.into(), PAtk::formula)
            .register_formula(AttackStat::MAtk.into(), MAtk::formula)
            .register_formula(AttackStat::PAtkSpd.into(), PAtkSpd::formula)
            .register_formula(AttackStat::Accuracy.into(), Accuracy::formula)
            .register_formula(AttackStat::CastSpd.into(), CastSpd::formula);
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct AttackStats(FloatStats<AttackStat>);

impl Default for AttackStats {
    fn default() -> Self {
        let base_class = BaseClass::default();
        let mut float_stats = FloatStats::default();
        for stat in AttackStat::iter() {
            float_stats.insert(stat, stat.default_value(base_class));
        }
        Self(float_stats)
    }
}

impl<'de> Deserialize<'de> for AttackStats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Use a raw Value to handle different types
        let partial: HashMap<AttackStat, serde_json::Value> = HashMap::deserialize(deserializer)?;
        let mut stats = AttackStats::default();

        for (stat, value) in partial {
            let float_value = if stat == AttackStat::EffectKind && value.is_string() {
                // If it's EffectKind and a string, deserialize it using the EffectKind deserializer
                let effect_kind: EffectKind =
                    serde_json::from_value(value.clone()).map_err(|e| {
                        serde::de::Error::custom(format!("Failed to parse EffectKind: {e}"))
                    })?;

                // Convert to f32
                <f32 as From<EffectKind>>::from(effect_kind)
            } else {
                // For all other stats, convert to f32
                value.as_f64().ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "Expected number for {stat:?}, got: {value:?}"
                    ))
                })? as f32
            };

            stats.insert(stat, float_value);
        }

        Ok(stats)
    }
}

impl AsRef<GenericStats<AttackStat, f32>> for AttackStats {
    fn as_ref(&self) -> &GenericStats<AttackStat, f32> {
        &self.0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum AttackStat {
    PAtk,
    PvpPAtkBonus,
    PSkillPower,
    PvpPSkillBonus,
    PAtkSpd,
    PvePAtkBonus,
    PveSkillBonus,
    PveBowPAtkBonus,
    PveBowSkillBonus,
    AttackReuse,
    PhysicalSkillReuse,
    RythmSkillReuse,
    MAtk,
    PvpMAtkBonus,
    PveMAtkBonus,
    CastSpd,
    MagicSkillReuse,
    Accuracy,
    PAtkRange,
    PAtkWidth,
    PAtkRandom,
    MAtkRange,
    AttackCountMax,
    EffectKind,
    SkillMastery,
    SkillMasteryRate,
}

impl StatTrait for AttackStat {
    fn default_value<V: StatValue>(&self, base_class: BaseClass) -> V {
        let value = match self {
            AttackStat::PAtk => base_class.base_p_atk(),
            AttackStat::MAtk => base_class.base_m_atk(),
            AttackStat::PAtkRange => PAtkRange::BASE as f32,
            AttackStat::PAtkWidth => PAtkWidth::BASE as f32,
            AttackStat::PAtkSpd => PAtkSpd::BASE as f32,
            AttackStat::CastSpd => CastSpd::BASE as f32,
            AttackStat::PveBowPAtkBonus => 1.0,
            AttackStat::PveBowSkillBonus => 1.0,
            AttackStat::PvePAtkBonus => 1.0,
            AttackStat::PveSkillBonus => 1.0,
            AttackStat::PvpPAtkBonus => 1.0,
            AttackStat::PvpPSkillBonus => 1.0,
            AttackStat::PvpMAtkBonus => 1.0,
            AttackStat::MagicSkillReuse => 1.0,
            AttackStat::PhysicalSkillReuse => 1.0,
            AttackStat::RythmSkillReuse => 1.0,
            _ => 0.0,
        };
        V::from(value).unwrap_or_default()
    }
}
