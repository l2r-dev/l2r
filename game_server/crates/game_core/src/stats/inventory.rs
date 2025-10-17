use crate::stats::{StatTrait, StatValue, UIntStats};
use bevy::{platform::collections::HashMap, prelude::*};
use derive_more::{From, Into};
use l2r_core::model::base_class::BaseClass;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

pub struct InventoryStatsComponentsPlugin;
impl Plugin for InventoryStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InventoryStats>()
            .register_type::<InventoryStat>()
            .register_type::<InventoryLimit>()
            .register_type::<WarehouseLimit>()
            .register_type::<FreightLimit>()
            .register_type::<PrivateSellLimit>()
            .register_type::<PrivateBuyLimit>()
            .register_type::<DwarfRecipeLimit>()
            .register_type::<CommonRecipeLimit>()
            .register_type::<WeightCurrent>()
            .register_type::<WeightLimit>()
            .register_type::<WeightPenalty>();
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct InventoryStats(UIntStats<InventoryStat>);
impl Default for InventoryStats {
    fn default() -> Self {
        let mut stats = HashMap::default();

        for stat in InventoryStat::iter() {
            let value = match stat {
                InventoryStat::InventoryLimit => InventoryLimit::default().into(),
                InventoryStat::WarehouseLimit => WarehouseLimit::default().into(),
                InventoryStat::FreightLimit => FreightLimit::default().into(),
                InventoryStat::PrivateSellLimit => PrivateSellLimit::default().into(),
                InventoryStat::PrivateBuyLimit => PrivateBuyLimit::default().into(),
                InventoryStat::DwarfRecipeLimit => DwarfRecipeLimit::default().into(),
                InventoryStat::CommonRecipeLimit => CommonRecipeLimit::default().into(),
                InventoryStat::WeightCurrent => WeightCurrent::default().into(),
                InventoryStat::WeightLimit => WeightLimit::default().into(),
                InventoryStat::WeightPenalty => WeightPenalty::default().into(),
            };
            stats.insert(stat, value);
        }
        Self(stats.into())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum InventoryStat {
    InventoryLimit,
    WarehouseLimit,
    FreightLimit,
    PrivateSellLimit,
    PrivateBuyLimit,
    DwarfRecipeLimit,
    CommonRecipeLimit,
    WeightCurrent,
    WeightLimit,
    WeightPenalty,
}

impl StatTrait for InventoryStat {
    fn default_value<V: super::StatValue>(&self, _base_class: BaseClass) -> V {
        V::default()
    }

    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::from(u32::MAX).unwrap_or_default()
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct InventoryLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct WarehouseLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct FreightLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct PrivateSellLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct PrivateBuyLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct DwarfRecipeLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct CommonRecipeLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct WeightCurrent(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct WeightLimit(u32);

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Reflect,
    Serialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
pub struct WeightPenalty(u32);
