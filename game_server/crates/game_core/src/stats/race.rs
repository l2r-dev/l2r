use crate::stats::BaseClassStats;
use bevy::{platform::collections::HashMap, prelude::*};
use l2r_core::model::{base_class::BaseClass, race::Race};
use serde::{Deserialize, Serialize};

#[derive(
    Asset, Deref, DerefMut, Clone, Debug, Default, Deserialize, Resource, Serialize, TypePath,
)]
pub struct RaceStats(HashMap<Race, HashMap<BaseClass, BaseClassStats>>);

impl RaceStats {
    pub fn get(&self, race: Race, base_class: BaseClass) -> &BaseClassStats {
        // Try to get the PrimalStats for the specific race
        if let Some(primal_stats_by_race) = self.0.get(&race) {
            // If found, try to get PrimalStats for the specific base_class
            if let Some(primal_stats) = primal_stats_by_race.get(&base_class) {
                return primal_stats;
            }
        }

        // If not found, get the PrimalStats for the default race
        let primal_stats_default_race = self
            .0
            .get(&Race::Human)
            .expect("Default race not found in map");

        // Get the PrimalStats for the specific base_class for the default race
        primal_stats_default_race
            .get(&base_class)
            .expect("Base class not found for default race")
    }
}

pub const RACE_STATS_ASSETS_ERROR: &str =
    "Failed to load RaceStats asset, must be loaded on startup";

#[derive(Default, Deref)]
pub struct RaceStatsHandle(Handle<RaceStats>);
impl From<Handle<RaceStats>> for RaceStatsHandle {
    fn from(handle: Handle<RaceStats>) -> Self {
        RaceStatsHandle(handle)
    }
}
