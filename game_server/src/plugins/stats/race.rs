use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use game_core::stats::{RaceStats, RaceStatsHandle, StatsTable};
use std::path::PathBuf;

pub struct RaceStatsPlugin;
impl Plugin for RaceStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<RaceStats>::new(&["json"]))
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

fn setup(asset_server: Res<AssetServer>, mut stats_table: ResMut<StatsTable>) {
    let mut path = PathBuf::new();
    path.push("race_stats");
    path.push(l2r_core::chronicles::CHRONICLE);
    path.push("race_stats");
    path.set_extension("json");
    stats_table.race_stats = RaceStatsHandle::from(asset_server.load(path));
}

fn update(
    race_stats_assets: Res<Assets<RaceStats>>,
    mut events: EventReader<AssetEvent<RaceStats>>,
    mut stats_table: ResMut<StatsTable>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if race_stats_assets.get(*id).is_some() {
                    bevy::log::debug!("RaceStats updated");
                }
            }
            AssetEvent::LoadedWithDependencies { id: _ } => {
                stats_table.init_race_stats();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::stats::RaceStats;
    use l2r_core::{assets::ASSET_DIR, utils::get_base_path};
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parsing_from_json() {
        let mut asset_dir = get_base_path();
        asset_dir.push(ASSET_DIR);

        let mut path = asset_dir;
        path.push("tests");
        path.push("race_stats");
        path.set_extension("json");

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));

        let reader = BufReader::new(file);

        let race_stats: RaceStats =
            from_reader(reader).unwrap_or_else(|_| panic!("Failed to parse from JSON: {:?}", path));

        assert!(!race_stats.is_empty());
    }
}
