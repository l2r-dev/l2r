use crate::plugins::{config::Config, state::LoadingSystems};
use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use sea_orm::{DynIden, prelude::SeaRc, sea_query::Alias};
use sea_orm_migration::{MigrationTrait, MigratorTrait, async_trait};

mod accounts_init;

use accounts_init::*;

pub struct LoginServerMigrationPlugin;
impl Plugin for LoginServerMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            LoginServerMigrator::init.in_set(LoadingSystems::Migration),
        );
    }
}

#[derive(Clone, Component, Copy, Default)]
pub struct MigrationTaskSpawned(bool);
impl MigrationTaskSpawned {
    pub fn complete(&self) -> bool {
        self.0
    }
}

pub struct LoginServerMigrator;
impl LoginServerMigrator {
    pub fn init(
        mut commands: Commands,
        task_flag: Query<(Entity, Ref<MigrationTaskSpawned>)>,
        mut state: ResMut<NextState<LoadingSystems>>,
    ) {
        if let Ok((entity, migration_task_flag)) = task_flag.single() {
            if migration_task_flag.0 {
                commands.entity(entity).despawn();
                log::debug!("Setting state to RepositoryInit");
                state.set(LoadingSystems::RepositoryInit);
            }
        } else {
            commands.spawn(MigrationTaskSpawned(false));
            commands.spawn_task(move || async move { Self::run().await });
        }
    }

    async fn run() -> Result<(), AccessError> {
        let database_url: String = AsyncWorld
            .resource::<Config>()
            .get(|config| config.general().database_url.clone().into())?;
        log::debug!("Connecting to database for migration: {}", database_url);

        let connection = match sea_orm::Database::connect(&database_url).await {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to connect to database for migration: {:?}", e);
                return Ok(());
            }
        };

        if let Err(e) = LoginServerMigrator::up(&connection, None).await {
            log::error!("Failed to run migrations: {:?}", e);
            return Ok(());
        }

        AsyncWorld
            .query_single::<Mut<MigrationTaskSpawned>>()
            .get_mut(|mut task| task.0 = true)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MigratorTrait for LoginServerMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(AccountsMigration)]
    }

    fn migration_table_name() -> DynIden {
        SeaRc::new(Alias::new("seaql_migrations_login"))
    }
}
