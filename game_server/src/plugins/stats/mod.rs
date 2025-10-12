use bevy::prelude::*;
use game_core::stats::*;

mod base;
mod calc;
mod class_tree;
mod race;

use base::*;
pub use calc::*;
use race::*;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatsComponentsPlugin);

        app.add_plugins(StatCalculationPlugin)
            .add_plugins(class_tree::ClassTreePlugin)
            .add_plugins(RaceStatsPlugin)
            .add_plugins(BaseStatsPlugin);
    }
}
