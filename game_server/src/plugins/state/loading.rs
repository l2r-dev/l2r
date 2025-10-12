use super::GameServerStateSystems;
use bevy::{log, prelude::*};
use game_core::{object_id::ObjectIdManager, stats::StatsTable};
use l2r_core::db::{DbConnection, RepositoryManager};
use scripting;
use state::{LoadingPlugin, LoadingSystems};

pub struct LoadingProcessPlugin;

impl Plugin for LoadingProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin);

        app.add_systems(
            Update,
            set_migration.in_set(LoadingSystems::DatabaseConnection),
        )
        .add_systems(Update, set_asset_init.in_set(LoadingSystems::IdInit))
        .add_systems(Update, set_id_init.in_set(LoadingSystems::RepositoryInit))
        .add_systems(
            Update,
            set_runtime_scripts_init.in_set(LoadingSystems::AssetInit),
        );

        app.add_systems(
            Update,
            set_run_state.in_set(LoadingSystems::RuntimeScriptsInit),
        );
    }
}

fn set_migration(
    mut state: ResMut<NextState<LoadingSystems>>,
    db_connection: Option<Res<DbConnection>>,
) {
    let Some(db_connection) = db_connection else {
        return;
    };

    if db_connection.is_mock() {
        log::debug!("Setting state to RepositoryInit (mock connection)");
        state.set(LoadingSystems::RepositoryInit);
        return;
    }

    if !db_connection.disconnected() {
        log::debug!("Setting state to Migration");
        state.set(LoadingSystems::Migration);
    }
}

fn set_id_init(
    mut state: ResMut<NextState<LoadingSystems>>,
    repo_manager: Res<RepositoryManager>,
    db_connection: Option<Res<DbConnection>>,
) {
    // Skip repository check for mock connections (tests)
    if let Some(db_conn) = db_connection
        && db_conn.is_mock()
    {
        state.set(LoadingSystems::IdInit);
        log::debug!("Setting state to IdInit (mock connection - skipping repository check)");
        return;
    }

    if !repo_manager.all_ready() {
        return;
    }

    state.set(LoadingSystems::IdInit);
    log::debug!("Setting state to IdInit");
}

fn set_asset_init(
    mut state: ResMut<NextState<LoadingSystems>>,
    object_id_manager: Option<Res<ObjectIdManager>>,
) {
    if object_id_manager.is_some() {
        log::debug!("Setting state to AssetInit");
        state.set(LoadingSystems::AssetInit);
    }
}

fn set_runtime_scripts_init(
    mut state: ResMut<NextState<LoadingSystems>>,
    stats_table: Res<StatsTable>,
) {
    if stats_table.initialized() {
        log::debug!("Setting state to RuntimeScriptsInit");
        state.set(LoadingSystems::RuntimeScriptsInit);
    }
}

fn set_run_state(
    mut state: ResMut<NextState<GameServerStateSystems>>,
    runtime_scripts_task: Query<&scripting::RuntimeScriptsTaskSpawned>,
) {
    if let Ok(task) = runtime_scripts_task.single()
        && task.loaded
    {
        log::debug!("Setting state to Run");
        state.set(GameServerStateSystems::Run);
    }
}
