use bevy::prelude::*;
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

mod evasion;
mod m_def;
mod p_def;
mod shield_angle;
mod shield_defence;
mod shield_rate;

use crate::{items::DollSlot, stats::*};
pub use evasion::*;
pub use m_def::*;
pub use p_def::*;
pub use shield_angle::*;
pub use shield_defence::*;
pub use shield_rate::*;

pub struct DefenceComponentsPlugin;
impl Plugin for DefenceComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DefenceStats>()
            .register_type::<DefenceStat>();

        app.world_mut()
            .resource_mut::<StatFormulaRegistry>()
            .register_formula(DefenceStat::PDef.into(), PDef::formula)
            .register_formula(DefenceStat::MDef.into(), MDef::formula)
            .register_formula(DefenceStat::Evasion.into(), Evasion::formula)
            .register_formula(DefenceStat::ShieldDefence.into(), ShieldDefence::formula)
            .register_formula(DefenceStat::ShieldRate.into(), ShieldRate::formula)
            .register_formula(DefenceStat::ShieldAngle.into(), ShieldAngle::formula);
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct DefenceStats(FloatStats<DefenceStat>);

impl Default for DefenceStats {
    fn default() -> Self {
        let base_class = BaseClass::default();
        let mut float_stats = FloatStats::default();

        for stat in DefenceStat::iter() {
            float_stats.insert(stat, stat.default_value(base_class));
        }

        Self(float_stats)
    }
}

impl AsRef<GenericStats<DefenceStat, f32>> for DefenceStats {
    fn as_ref(&self) -> &GenericStats<DefenceStat, f32> {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DefenceStats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let partial: HashMap<DefenceStat, f32> = HashMap::deserialize(deserializer)?;
        let mut stats = DefenceStats::default();
        // Merge with deserialized values
        for (stat, value) in partial {
            stats.insert(stat, value);
        }
        Ok(stats)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum DefenceStat {
    PDef,
    PvpPDefBonus,
    MDef,
    PvpMDefBonus,
    ShieldDefence,
    ShieldRate,
    ShieldAngle,
    Evasion,
    PSkillEvasion,
    PvpPSkillBonus,
    DefenceCriticalRate,
    DefenceCriticalRateAdditional,
    DefenceCriticalDamage,
    DefenceCriticalDamageAdditional,
    DamageZoneVulnerability,
    MovementVulnerability,
    CancelVulnerability,
    DebuffVulnerability,
    BuffVulnerability,
    FireResistance,
    WindResistance,
    WaterResistance,
    EarthResistance,
    HolyResistance,
    DarkResistance,
    MagicSuccessResistance,
    DebuffImmunity,
    CancelProficiency,
    ReflectDamagePercent,
    ReflectSkillMagic,
    ReflectSkillPhysical,
    VengeanceSkillMagicDamage,
    VengeanceSkillPhysicalDamage,
    AbsorbDamagePercent,
    TransferDamagePercent,
    ManaShieldPercent,
    TransferDamageToPlayer,
    AbsorbManaDamagePercent,
}

impl StatTrait for DefenceStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        let value = match self {
            DefenceStat::PDef => DollSlot::base_p_def_total(),
            DefenceStat::MDef => DollSlot::base_m_def_total(),
            DefenceStat::DefenceCriticalRate => 1.0,
            DefenceStat::DefenceCriticalRateAdditional => 0.0,
            DefenceStat::DefenceCriticalDamage => 1.0,
            DefenceStat::DefenceCriticalDamageAdditional => 0.0,
            DefenceStat::ShieldAngle => ShieldAngle::BASE as f32,
            DefenceStat::ShieldRate => ShieldRate::BASE,
            _ => 0.0,
        };
        V::from(value).unwrap_or_default()
    }

    /// When equipment is present in a slot, its defence contribution is handled by the StatModifiers
    /// system, which applies the item's actual defence values.
    /// If not - we use the base defence value from the character's base class.
    fn with_doll<V: StatValue>(&self, base_class: BaseClass, paper_doll: &PaperDoll) -> V {
        let value = match self {
            DefenceStat::PDef => DollSlot::base_p_def_slots()
                .into_iter()
                .filter(|&slot| paper_doll.get(slot).is_none())
                .fold(0.0, |acc, slot| acc + slot.base_p_def()),

            DefenceStat::MDef => DollSlot::base_m_def_slots()
                .into_iter()
                .filter(|&slot| paper_doll.get(slot).is_none())
                .fold(0.0, |acc, slot| acc + slot.base_m_def()),
            _ => self.default_value(base_class),
        };
        V::from(value).unwrap_or_default()
    }
}
