use crate::stats::{
    CLASS_TREE_ASSETS_ERROR, ClassId, ClassTree, ClassTreeHandle, Level, LeveledVitalsStats,
    RACE_STATS_ASSETS_ERROR, RaceStats, RaceStatsHandle, VitalsStats, VitalsStatsHandlers,
};
use bevy::prelude::*;
use bevy_ecs::system::SystemParam;

#[derive(Debug, Default)]
pub struct StatsAssetLoadingState {
    pub class_tree: bool,
    pub race_stats: bool,
    pub vitals_stats: bool,
}

#[derive(Default, Resource)]
pub struct StatsTable {
    initialized: StatsAssetLoadingState,
    pub race_stats: RaceStatsHandle,
    pub vitals_stats: VitalsStatsHandlers,
    pub class_tree: ClassTreeHandle,
}

impl StatsTable {
    pub fn init_class_tree(&mut self) {
        self.initialized.class_tree = true;
    }

    pub fn init_race_stats(&mut self) {
        self.initialized.race_stats = true;
    }

    pub fn init_vitals_stats(&mut self) {
        self.initialized.vitals_stats = true;
    }

    pub fn initialized(&self) -> bool {
        self.initialized.class_tree && self.initialized.race_stats && self.initialized.vitals_stats
    }

    pub fn race_stats<'a>(&'a self, assets: &'a Assets<RaceStats>) -> &'a RaceStats {
        assets
            .get(self.race_stats.id())
            .expect(RACE_STATS_ASSETS_ERROR)
    }

    pub fn race_stats_world<'a>(&'a self, world: &'a World) -> &'a RaceStats {
        let race_stats_assets = world.resource::<Assets<RaceStats>>();

        self.race_stats(race_stats_assets)
    }

    pub fn class_tree<'a>(&'a self, assets: &'a Assets<ClassTree>) -> &'a ClassTree {
        assets
            .get(self.class_tree.id())
            .expect(CLASS_TREE_ASSETS_ERROR)
    }

    pub fn class_tree_world<'a>(&'a self, world: &'a World) -> &'a ClassTree {
        let class_tree_assets = world.resource::<Assets<ClassTree>>();

        class_tree_assets
            .get(self.class_tree.id())
            .expect(CLASS_TREE_ASSETS_ERROR)
    }

    pub fn vitals_stats<'a>(
        &'a self,
        assets: &'a Assets<LeveledVitalsStats>,
        class_id: ClassId,
        level: Level,
    ) -> Option<&'a VitalsStats> {
        self.vitals_stats.get(&class_id).and_then(|asset_handler| {
            assets
                .get(asset_handler)
                .and_then(|vitals_stats| vitals_stats.get(&level))
        })
    }

    pub fn vitals_stats_world<'a>(
        &'a self,
        world: &'a World,
        class_id: ClassId,
        level: Level,
    ) -> Option<&'a VitalsStats> {
        let vitals_stats_assets = world.resource::<Assets<LeveledVitalsStats>>();

        self.vitals_stats(vitals_stats_assets, class_id, level)
    }
}

#[derive(SystemParam)]
pub struct StatsTableQuery<'w> {
    pub stats_table: Res<'w, StatsTable>,
    pub class_tree_assets: Res<'w, Assets<ClassTree>>,
    pub race_stats_assets: Res<'w, Assets<RaceStats>>,
    pub vitals_stats_assets: Res<'w, Assets<LeveledVitalsStats>>,
}

impl StatsTableQuery<'_> {
    pub fn class_tree(&self) -> &ClassTree {
        self.stats_table.class_tree(self.class_tree_assets.as_ref())
    }

    pub fn race_stats(&self) -> &RaceStats {
        self.stats_table.race_stats(self.race_stats_assets.as_ref())
    }

    pub fn vitals_stats(&self, class_id: ClassId, level: Level) -> Option<&VitalsStats> {
        self.stats_table
            .vitals_stats(self.vitals_stats_assets.as_ref(), class_id, level)
    }
}
