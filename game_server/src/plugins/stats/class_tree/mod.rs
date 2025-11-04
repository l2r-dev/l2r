use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use game_core::stats::{ClassTree, StatsTable};
use state::LoadingSystems;

mod sub_class;

pub(crate) struct ClassTreePlugin;
impl Plugin for ClassTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ClassTree>::new(&["ron"]))
            .add_plugins(sub_class::SubClassStatsPlugin);

        app.add_systems(
            Update,
            (
                load_assets.in_set(LoadingSystems::AssetInit),
                update_stats_table.in_set(LoadingSystems::AssetInit),
            ),
        );
    }
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut stats_table: ResMut<StatsTable>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let class_tree_handle = asset_server.load("class_tree.ron");
    *stats_table.class_tree = class_tree_handle;
    *loaded = true;
}

pub fn update_stats_table(
    mut stats_table: ResMut<StatsTable>,
    mut events: EventReader<AssetEvent<ClassTree>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id: _ } = event {
            stats_table.init_class_tree();
            debug!("Class tree stats updated from asset");
        }
    }
}
