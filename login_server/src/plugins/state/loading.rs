use super::LoginStateSystems;
use bevy::{log, prelude::*};
use l2r_core::db::DbConnection;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingSystems>();

        app.configure_sets(
            Update,
            LoadingSystems::DatabaseConnection
                .in_set(LoginStateSystems::Load)
                .run_if(in_state(LoadingSystems::DatabaseConnection)),
        )
        .configure_sets(
            FixedUpdate,
            LoadingSystems::DatabaseConnection.run_if(in_state(LoadingSystems::DatabaseConnection)),
        )
        .configure_sets(
            Update,
            LoadingSystems::Migration
                .in_set(LoginStateSystems::Load)
                .run_if(in_state(LoadingSystems::Migration)),
        )
        .configure_sets(
            FixedUpdate,
            LoadingSystems::Migration
                .before(LoadingSystems::RepositoryInit)
                .run_if(in_state(LoadingSystems::Migration)),
        )
        .configure_sets(
            Update,
            LoadingSystems::RepositoryInit
                .in_set(LoginStateSystems::Load)
                .run_if(in_state(LoadingSystems::RepositoryInit)),
        )
        .configure_sets(
            FixedUpdate,
            LoadingSystems::RepositoryInit
                .in_set(LoginStateSystems::Load)
                .run_if(in_state(LoadingSystems::RepositoryInit)),
        );

        app.add_systems(
            Update,
            set_migration.in_set(LoadingSystems::DatabaseConnection),
        );
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States, SystemSet)]
pub enum LoadingSystems {
    #[default]
    DatabaseConnection,
    Migration,
    RepositoryInit,
}

fn set_migration(
    mut state: ResMut<NextState<LoadingSystems>>,
    db_connection: Option<Res<DbConnection>>,
) {
    let Some(db_connection) = db_connection else {
        return;
    };

    if !db_connection.disconnected() {
        log::debug!("Setting state to Migration");
        state.set(LoadingSystems::Migration);
    }
}
