use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use game_core::stats::{ClassTree, StatsTable};

mod sub_class;

pub(crate) struct ClassTreePlugin;
impl Plugin for ClassTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ClassTree>::new(&["ron"]))
            .add_plugins(sub_class::SubClassStatsPlugin);

        app.add_systems(Startup, load_assets)
            .add_systems(Update, update_stats_table);
    }
}

pub fn load_assets(asset_server: Res<AssetServer>, mut stats_table: ResMut<StatsTable>) {
    let class_tree_handle = asset_server.load("class_tree.ron");
    *stats_table.class_tree = class_tree_handle;
}

pub fn update_stats_table(
    mut stats_table: ResMut<StatsTable>,
    mut events: EventReader<AssetEvent<ClassTree>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if stats_table.class_tree.id() == *id {
                    bevy::log::debug!("Class tree asset updated");
                }
            }
            AssetEvent::LoadedWithDependencies { id: _ } => {
                stats_table.init_class_tree();
            }
            _ => {}
        }
    }
}
