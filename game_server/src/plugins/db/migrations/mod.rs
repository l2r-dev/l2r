use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use config::Config;
use sea_orm::{DynIden, prelude::SeaRc, sea_query::Alias};
use sea_orm_migration::{MigrationTrait, MigratorTrait, async_trait};
use state::LoadingSystems;

mod character_shortcuts_init;
mod characters_init;
mod characters_skills_init;
mod items_init;

use character_shortcuts_init::*;
use characters_init::*;
use characters_skills_init::*;
use items_init::*;

pub struct GameServerMigrationPlugin;
impl Plugin for GameServerMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            GameServerMigrator::init.in_set(LoadingSystems::Migration),
        );
    }
}

#[derive(Clone, Component, Copy, Default)]
pub struct MigrationTaskSpawned {
    pub finished: bool,
}

pub struct GameServerMigrator;
impl GameServerMigrator {
    pub fn init(
        mut commands: Commands,
        task_flag: Query<(Entity, Ref<MigrationTaskSpawned>)>,
        mut state: ResMut<NextState<LoadingSystems>>,
    ) {
        if let Ok((entity, migration_task_flag)) = task_flag.single() {
            if migration_task_flag.finished {
                commands.entity(entity).despawn();
                log::debug!("Setting state to RepositoryInit");
                state.set(LoadingSystems::RepositoryInit);
            }
        } else {
            commands.spawn(MigrationTaskSpawned::default());
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

        if let Err(e) = GameServerMigrator::up(&connection, None).await {
            log::error!("Failed to run migrations: {:?}", e);
            return Ok(());
        }

        AsyncWorld
            .query_single::<Mut<MigrationTaskSpawned>>()
            .get_mut(|mut task| task.finished = true)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MigratorTrait for GameServerMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CharactersMigration),
            Box::new(ItemsMigration),
            Box::new(CharacterShortcutsMigration),
            Box::new(CharacterSkillsMigration),
        ]
    }

    fn migration_table_name() -> DynIden {
        SeaRc::new(Alias::new("seaql_migrations_game"))
    }
}
