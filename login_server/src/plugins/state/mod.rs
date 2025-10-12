use bevy::{log, prelude::*};
use l2r_core::db::{DbConnection, RepositoryManager};

mod loading;

use crate::plugins::db::migrations::MigrationTaskSpawned;
pub use loading::*;

pub struct LoginStateSystemsPlugin;
impl Plugin for LoginStateSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoginStateSystems>();

        app.configure_sets(
            Update,
            (
                LoginStateSystems::Load.run_if(in_state(LoginStateSystems::Load)),
                LoginStateSystems::Run.run_if(in_state(LoginStateSystems::Run)),
            ),
        )
        .configure_sets(
            FixedUpdate,
            (
                LoginStateSystems::Load.run_if(in_state(LoginStateSystems::Load)),
                LoginStateSystems::Run.run_if(in_state(LoginStateSystems::Run)),
            ),
        );

        app.add_plugins(LoadingPlugin);

        app.add_systems(Update, set_run_state.in_set(LoginStateSystems::Load));
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States, SystemSet)]
pub enum LoginStateSystems {
    #[default]
    Load,
    Run,
}

fn set_run_state(
    mut state: ResMut<NextState<LoginStateSystems>>,
    repo_manager: Res<RepositoryManager>,
    migrations: Query<Ref<MigrationTaskSpawned>>,
    db_conn: Res<DbConnection>,
) -> Result<()> {
    if db_conn.disconnected() {
        return Ok(());
    }

    let migrations_task = migrations.single()?;
    if !migrations_task.complete() {
        return Ok(());
    }

    if !repo_manager.all_ready() {
        return Ok(());
    }

    state.set(LoginStateSystems::Run);
    log::debug!("Setting state to Run");
    Ok(())
}
