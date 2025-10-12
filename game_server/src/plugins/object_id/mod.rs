use bevy::{log, prelude::*};
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use game_core::{
    character, items,
    object_id::{ObjectId, ObjectIdComponentsPlugin, ObjectIdManager, ObjectIdManagerTaskSpawned},
};
use l2r_core::db::{DbConnection, Repository, RepositoryManager, TypedRepositoryManager};
use state::LoadingSystems;

pub struct ObjectIdPlugin;
impl Plugin for ObjectIdPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ObjectIdComponentsPlugin);

        app.add_systems(
            Update,
            init_object_id_manager.in_set(LoadingSystems::IdInit),
        );
    }
}

fn init_object_id_manager(
    mut commands: Commands,
    repo_manager: Res<RepositoryManager>,
    object_id_manager: Option<Res<ObjectIdManager>>,
    task_flag: Query<Entity, With<ObjectIdManagerTaskSpawned>>,
    db_connection: Res<DbConnection>,
) -> Result<()> {
    // If we're using a mock database connection, ensure an ObjectIdManager is always present
    if db_connection.is_mock() {
        if object_id_manager.is_none() {
            log::trace!("Initializing default ObjectIdManager (mock connection)");
            commands.insert_resource(ObjectIdManager::new());
        }
        return Ok(());
    }

    let character_repo = repo_manager.typed::<ObjectId, character::model::Entity>()?;
    let items_repo = repo_manager.typed::<ObjectId, items::model::Entity>()?;

    if let Ok(entity) = task_flag.single() {
        if object_id_manager.is_some() {
            commands.entity(entity).despawn();
        }
    } else if object_id_manager.is_none() {
        // Spawn an entity to track that the task is running
        commands.spawn(ObjectIdManagerTaskSpawned);
        // Spawn a task to load object IDs from the database
        commands.spawn_task(move || async move {
            let mut object_ids = character_repo
                .list_column::<i32>(character::model::Column::Id)
                .await
                .unwrap_or_default();

            object_ids.extend(
                items_repo
                    .list_column::<i32>(items::model::Column::ObjectId)
                    .await
                    .unwrap_or_default(),
            );

            log::info!("Loaded {} object IDs from database.", object_ids.len());
            let object_id_manager = ObjectIdManager::prepare_occupied(&object_ids);

            AsyncWorld.apply_command(move |world: &mut World| {
                world.insert_resource(object_id_manager);
            });

            Ok(())
        });
    }
    Ok(())
}
