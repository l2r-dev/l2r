use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use game_core::stats::{RaceStats, RaceStatsHandle, StatsTable};
use state::LoadingSystems;
use std::path::PathBuf;

pub struct RaceStatsPlugin;
impl Plugin for RaceStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<RaceStats>::new(&["json"]))
            .add_systems(
                Update,
                (
                    setup.in_set(LoadingSystems::AssetInit),
                    update.in_set(LoadingSystems::AssetInit),
                ),
            );
    }
}

fn setup(
    asset_server: Res<AssetServer>,
    mut stats_table: ResMut<StatsTable>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let mut path = PathBuf::new();
    path.push("race_stats");
    path.push(l2r_core::chronicles::CHRONICLE);
    path.push("race_stats");
    path.set_extension("json");
    stats_table.race_stats = RaceStatsHandle::from(asset_server.load(path));
    *loaded = true;
}

fn update(mut events: EventReader<AssetEvent<RaceStats>>, mut stats_table: ResMut<StatsTable>) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id: _ } = event {
            stats_table.init_race_stats();
            debug!("Race stats updated from asset");
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
