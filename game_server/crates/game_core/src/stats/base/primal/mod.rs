use bevy::{log, prelude::*};
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strum::EnumIter;

mod constitution;
mod dexterity;
mod intelligence;
mod mental;
mod strenght;
mod wisdom;

use crate::stats::{GenericStats, StatTrait, StatValue, UIntStats};
pub use constitution::*;
pub use dexterity::*;
pub use intelligence::*;
pub use mental::*;
pub use strenght::*;
pub use wisdom::*;

pub struct PrimalStatsComponentsPlugin;
impl Plugin for PrimalStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PrimalStats>()
            .register_type::<PrimalStat>()
            .register_type::<STR>()
            .register_type::<DEX>()
            .register_type::<CON>()
            .register_type::<INT>()
            .register_type::<WIT>()
            .register_type::<MEN>();
    }
}

pub trait PrimalStatTrait {
    fn bonus(&self) -> f32;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum PrimalStat {
    #[serde(alias = "str")]
    STR,
    #[serde(alias = "con")]
    CON,
    #[serde(alias = "dex")]
    DEX,
    #[serde(alias = "int")]
    INT,
    #[serde(alias = "wit")]
    WIT,
    #[serde(alias = "men")]
    MEN,
}

impl StatTrait for PrimalStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::from(10).unwrap_or_default()
    }

    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::from(100).unwrap_or_default()
    }
}

#[derive(
    Clone, Component, Debug, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize, Default,
)]
pub struct PrimalStats(UIntStats<PrimalStat>);

impl AsRef<GenericStats<PrimalStat, u32>> for PrimalStats {
    fn as_ref(&self) -> &GenericStats<PrimalStat, u32> {
        &self.0
    }
}

pub trait PrimalBonusTable: Sized {
    fn calculate(value: u32) -> f32;
    fn table() -> &'static LazyLock<[f32; 101]>;

    fn bonus(value: u32) -> f32 {
        Self::table()
            .get(value as usize)
            .copied()
            .unwrap_or_else(|| {
                log::warn!(
                    "PrimalBonusTable::bonus: value {} out of bounds, calculating dynamically",
                    value
                );
                Self::calculate(value)
            })
    }
}
