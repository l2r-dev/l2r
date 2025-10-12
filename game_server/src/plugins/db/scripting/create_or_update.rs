use crate::plugins::db::GameRepoModel;
use game_core::{
    character::{self, skills::SkillPK},
    items,
    object_id::ObjectId,
    shortcut::model::ShortcutPK,
};
use l2r_core::{
    db::{Repository, RepositoryModel, TypedRepositoryManager},
    utils::block_on,
};
use scripting::{
    bindings::{FunctionCallContext, InteropError, ReflectReference},
    prelude::ScriptValue,
    utils::{ScriptValueToArguments, ValueWithOptional},
};

pub(crate) fn script_create_or_update(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;
    let args = ValueWithOptional::from_script_value(&data)?;

    let model_ref = match args.value {
        ScriptValue::Reference(ref_val) => ref_val,
        _ => {
            return Err(InteropError::value_mismatch(
                std::any::TypeId::of::<ReflectReference>(),
                args.value.clone(),
            ));
        }
    };

    // TODO: Parse _on_conflict_value to create custom OnConflict from scripts
    // now using default on_conflict from the model
    let _on_conflict_value = args.optional_value;

    let downcasted_model = GameRepoModel::try_from((model_ref, world_guard.clone()))?;
    world_guard.with_global_access(|world| {
        use l2r_core::db::RepositoryManager;

        let registry = world.resource::<RepositoryManager>();
        match downcasted_model {
            GameRepoModel::Character(character_model) => {
                let repo = registry.typed_interop::<ObjectId, character::model::Entity>()?;
                block_on(|| async move {
                    repo.create_or_update(&character_model, character::model::Model::on_conflict())
                        .await
                })?;
                Ok(true.into())
            }
            GameRepoModel::CharacterSkills(skill_model) => {
                let repo = registry.typed_interop::<SkillPK, character::skills::Entity>()?;
                block_on(|| async move {
                    repo.create_or_update(&skill_model, character::skills::Model::on_conflict())
                        .await
                })?;
                Ok(true.into())
            }
            GameRepoModel::Items(item_model) => {
                let repo = registry.typed_interop::<ObjectId, items::model::Entity>()?;
                block_on(|| async move {
                    repo.create_or_update(&item_model, items::model::Model::on_conflict())
                        .await
                })?;
                Ok(true.into())
            }
            GameRepoModel::CharacterShortcuts(shortcut_model) => {
                let repo =
                    registry.typed_interop::<ShortcutPK, game_core::shortcut::model::Entity>()?;
                block_on(|| async move {
                    repo.create_or_update(
                        &shortcut_model,
                        game_core::shortcut::model::Model::on_conflict(),
                    )
                    .await
                })?;
                Ok(true.into())
            }
        }
    })?
}
