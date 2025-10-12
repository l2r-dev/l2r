use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod level;
mod level_exp_data;
mod stats_core;
mod stats_rates;

pub use level::*;
pub use stats_core::*;
pub use stats_rates::*;

pub struct ProgressStatsComponentsPlugin;
impl Plugin for ProgressStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ProgressStats>()
            .register_type::<ProgressStat>()
            .register_type::<ProgressRatesStats>()
            .register_type::<ProgressRatesStat>()
            .register_type::<Level>()
            .register_type::<Exp>()
            .register_type::<Sp>();
    }
}

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
pub struct ProgressReward {
    #[serde(default)]
    pub exp: Exp,
    #[serde(default)]
    pub sp: Sp,
}
