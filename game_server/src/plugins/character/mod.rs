use crate::plugins::shutdown::ShutdownState;
use bevy::{log, prelude::*};
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use game_core::{
    character::{
        self, Character, CharacterComponentsPlugin, CharacterRepository, CharacterSave,
        model::{self, ModelUpdate},
    },
    items::ItemsQuery,
    object_id::ObjectId,
};
use l2r_core::{
    db::{DbError, Repository, RepositoryManager, TypedRepositoryManager},
    plugins::custom_hierarchy::*,
};
use sea_orm::{ColumnTrait, QueryFilter, UpdateResult, prelude::Expr};
use std::sync::atomic::Ordering;
use uuid::Uuid;

mod creation_menu;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CharacterComponentsPlugin);

        app.add_plugins(creation_menu::CharacterCreationPlugin);

        app.add_observer(save_char_to_database);

        app.add_systems(Update, sort_entities_into_folders);
    }
}

async fn reset_last_active_character(
    account_id: Uuid,
    character_repository: &CharacterRepository,
) -> Result<UpdateResult, DbError> {
    character_repository
        .update_many(|update| {
            update
                .col_expr(model::Column::IsLastActive, Expr::value(false))
                .filter(model::Column::AccountId.eq(account_id))
        })
        .await
}

fn sort_entities_into_folders(
    changed_children: Query<(Ref<Character>, Ref<DespawnChildren>), Changed<DespawnChildren>>,
    refs: Query<EntityRef>,
    mut commands: Commands,
) -> Result<()> {
    for (character, despawn_children) in changed_children.iter() {
        for child_entity in despawn_children.iter() {
            let entity_ref = refs.get(child_entity)?;
            commands.insert_into_folders(entity_ref, character.as_ref());
        }
    }
    Ok(())
}

fn save_char_to_database(
    save: Trigger<CharacterSave>,
    mut commands: Commands,
    characters: Query<(character::Query, Ref<DespawnChildOf>)>,
    items_query: ItemsQuery,
    mut chars_tables: Query<Mut<character::Table>>,
    repo_manager: Res<RepositoryManager>,
) -> Result<()> {
    if repo_manager.is_mock() {
        return Ok(());
    }
    let character_entity = save.target();
    let character_repository = repo_manager.typed::<ObjectId, character::model::Entity>()?;
    let (character, session) = characters.get(character_entity)?;
    let mut chars_table = chars_tables.get_mut(**session)?;
    chars_table.update_bundle(&character, &items_query);
    let char_name = character.name.to_string().clone();

    let char_id = *character.object_id;
    let model_update = ModelUpdate::from(&character);

    commands.spawn_task(move || async move {
        let char_model = character_repository.find_by_id(char_id).await?;

        let char_model = match char_model {
            Some(model) => model,
            None => {
                log::warn!("No character model found for {}, ID {}", char_name, char_id);
                return Ok(());
            }
        };

        reset_last_active_character(char_model.account_id, &character_repository).await?;

        character_repository
            .update(&char_model.update(model_update))
            .await?;

        AsyncWorld.resource::<ShutdownState>().get_mut(|state| {
            if state.shutdown_requested.load(Ordering::Acquire) {
                state.saved_chars += 1;
            }
        })
    });
    Ok(())
}
