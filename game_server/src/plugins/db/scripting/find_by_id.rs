use crate::plugins::db::{GameRepoKey, GameRepoName};
use bevy::prelude::*;
use game_core::{
    character::{self, skills::SkillPK},
    items,
    object_id::ObjectId,
    shortcut::model::ShortcutPK,
};
use l2r_core::{
    db::{Repository, TypedRepositoryManager},
    utils::{AllocatedReflectExt, block_on},
};
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::ScriptValue,
    utils::{ScriptValueToArguments, StringWithValue},
};
use std::str::FromStr;

pub(crate) fn script_find_by_id(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let args = StringWithValue::from_script_value(&data)?;
    let repo_name_str = args.string;
    let key_value = args.value;
    let repo_name = GameRepoName::from_str(&repo_name_str)
        .map_err(|_e| InteropError::invariant(format!("Unknown repository: {}", repo_name_str)))?;
    let repository_key = GameRepoKey::try_from((&repo_name, key_value))?;

    let world_guard = ctx.world()?;
    world_guard.with_global_access(|world| {
        use l2r_core::db::RepositoryManager;

        let repo_manager = world.resource::<RepositoryManager>();

        Ok(match repository_key {
            GameRepoKey::Character(object_id) => repo_manager
                .typed::<ObjectId, character::model::Entity>()
                .map(|repo| {
                    block_on(|| async move { repo.find_by_id(object_id).await })
                        .map(|model| world.new_allocated(model))
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            GameRepoKey::CharacterSkills(skill_pk) => repo_manager
                .typed::<SkillPK, character::skills::Entity>()
                .map(|repo| {
                    block_on(|| async move { repo.find_by_id(skill_pk).await })
                        .map(|model| world.new_allocated(model))
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            GameRepoKey::Items(object_id) => repo_manager
                .typed::<ObjectId, items::model::Entity>()
                .map(|repo| {
                    block_on(|| async move { repo.find_by_id(object_id).await })
                        .map(|model| world.new_allocated(model))
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            GameRepoKey::CharacterShortcuts(shortcut_pk) => repo_manager
                .typed::<ShortcutPK, game_core::shortcut::model::Entity>()
                .map(|repo| {
                    block_on(|| async move { repo.find_by_id(shortcut_pk).await })
                        .map(|model| world.new_allocated(model))
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
        })
    })?
}
