use crate::stats::*;
use l2r_core::model::base_class::BaseClass;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, FromRepr};

mod accuracy;
mod cast_spd;
mod effect_kind;
mod m_attack;
mod p_atk_max_targets_count;
mod p_atk_range;
mod p_atk_spd;
mod p_attack;
mod p_attack_width;

pub use accuracy::*;
pub use cast_spd::*;
pub use effect_kind::*;
pub use m_attack::*;
pub use p_atk_max_targets_count::*;
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
            .register_type::<PAtkRange>()
            .register_type::<PAtkMaxTargetsCount>();

        app.world_mut()
            .resource_mut::<StatFormulaRegistry>()
            .register_formula(AttackStat::PAtk.into(), PAtk::formula)
            .register_formula(AttackStat::MAtk.into(), MAtk::formula)
            .register_formula(AttackStat::PAtkSpd.into(), PAtkSpd::formula)
            .register_formula(AttackStat::Accuracy.into(), Accuracy::formula)
            .register_formula(AttackStat::CastSpd.into(), CastSpd::formula);
    }
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct AttackStats(FloatStats<AttackStat>);

impl<'de> Deserialize<'de> for AttackStats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct AttackStatsVisitor;

        impl<'de> Visitor<'de> for AttackStatsVisitor {
            type Value = AttackStats;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of attack stat variants to values")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut stats = AttackStats::default();

                while let Some(key) = map.next_key::<AttackStat>()? {
                    let float_value = if key == AttackStat::EffectKind {
                        // Try to deserialize as string (EffectKind variant name) first
                        let value: serde_json::Value = map.next_value()?;
                        if value.is_string() {
                            let effect_kind: EffectKind = serde_json::from_value(value.clone())
                                .map_err(|e| {
                                    serde::de::Error::custom(format!(
                                        "Failed to parse EffectKind: {e}"
                                    ))
                                })?;
                            <f32 as From<EffectKind>>::from(effect_kind)
                        } else {
                            // If it's already a number, just use it
                            value.as_f64().ok_or_else(|| {
                                serde::de::Error::custom(format!(
                                    "Expected number or string for EffectKind, got: {value:?}"
                                ))
                            })? as f32
                        }
                    } else {
                        // For all other stats, deserialize as f32
                        map.next_value::<f32>()?
                    };

                    stats.insert(key, float_value);
                }

                Ok(stats)
            }
        }

        deserializer.deserialize_map(AttackStatsVisitor)
    }
}

impl AsRef<GenericStats<AttackStat, f32>> for AttackStats {
    fn as_ref(&self) -> &GenericStats<AttackStat, f32> {
        &self.0
    }
}

#[repr(usize)]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumCount,
    FromRepr,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    TryFromPrimitive,
    IntoPrimitive,
)]
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
    PAtkMaxTargetsCount,
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
            AttackStat::PAtkMaxTargetsCount => 1.0,

            _ => 0.0,
        };
        V::from(value).unwrap_or_default()
    }

    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        let value = match self {
            AttackStat::PAtkSpd => PAtkSpd::MAX as f32,
            AttackStat::CastSpd => CastSpd::MAX as f32,
            AttackStat::EffectKind => EffectKind::MAX as f32,
            _ => f32::MAX,
        };
        V::from(value).unwrap_or_default()
    }
}
