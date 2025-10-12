use super::GameServerStateSystems;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingSystems>();

        configure_loading_sets(app, Update);
        configure_loading_sets(app, FixedUpdate);
    }
}
fn configure_loading_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            LoadingSystems::DatabaseConnection
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::DatabaseConnection)),
            LoadingSystems::Migration
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::Migration)),
            LoadingSystems::RepositoryInit
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::RepositoryInit)),
            LoadingSystems::IdInit
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::IdInit)),
            LoadingSystems::AssetInit
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::AssetInit)),
            LoadingSystems::RuntimeScriptsInit
                .in_set(GameServerStateSystems::Load)
                .run_if(in_state(LoadingSystems::RuntimeScriptsInit)),
        ),
    );
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States, SystemSet)]
pub enum LoadingSystems {
    #[default]
    DatabaseConnection,
    Migration,
    RepositoryInit,
    IdInit,
    AssetInit,
    RuntimeScriptsInit,
}
