use bevy::prelude::*;
use game_core::{
    abnormal_effects::{
        AbnormalEffect, AbnormalEffects, AbnormalEffectsChangeTracker, AbnormalEffectsTimers,
    },
    skills,
};
use l2r_core::utils::AllocatedReflectExt;
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::{NamespaceBuilder, ScriptValue},
};

pub fn register_script_functions(app: &mut App) {
    NamespaceBuilder::<AbnormalEffect>::new(app.world_mut())
        .register("add", script_add)
        .register("has_effect", script_has_effect)
        .register("remove", script_remove)
        .register("diff", script_diff);
}

fn script_add(
    ctx: FunctionCallContext,
    script_value: ScriptValue,
) -> Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let (entity, abnormal_effect) = match script_value {
        // it must be a list of two items: entity and abnormal effect
        ScriptValue::List(list) if list.len() == 2 => {
            let entity = list
                .first()
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
            let abnormal_effect = list
                .get(1)
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

            if let (ScriptValue::Reference(entity_ref), ScriptValue::Reference(effect_ref)) =
                (entity, abnormal_effect)
            {
                let entity = entity_ref.downcast::<Entity>(world_guard.clone())?;
                let abnormal_effect = effect_ref.downcast::<AbnormalEffect>(world_guard.clone())?;
                (entity, abnormal_effect)
            } else {
                return Err(InteropError::type_mismatch(
                    std::any::TypeId::of::<Entity>(),
                    None,
                ));
            }
        }
        _ => return Err(InteropError::argument_count_mismatch(2, 1)),
    };

    world_guard.with_global_access(|world| {
        if let Ok(mut entity_world_mut) = world.get_entity_mut(entity) {
            if let Some(mut abnormal_effects) = entity_world_mut.get_mut::<AbnormalEffects>() {
                abnormal_effects.add(abnormal_effect);
                ScriptValue::Integer(1)
            } else {
                ScriptValue::Integer(0) // Entity doesn't have AbnormalEffects component
            }
        } else {
            ScriptValue::Integer(0) // Entity not found
        }
    })
}

fn script_diff(
    ctx: FunctionCallContext,
    script_value: ScriptValue,
) -> Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let entity = match script_value {
        ScriptValue::Reference(entity_ref) => entity_ref.downcast::<Entity>(world_guard.clone())?,
        _ => {
            return Err(InteropError::type_mismatch(
                std::any::TypeId::of::<Entity>(),
                None,
            ));
        }
    };

    world_guard.with_global_access(|world| {
        if let Ok(entity_world_ref) = world.get_entity(entity) {
            if let Some(tracker) = entity_world_ref.get::<AbnormalEffectsChangeTracker>() {
                let added = tracker.added().to_vec();
                let removed = tracker.removed().to_vec();
                let added_list = world.new_allocated(Some(added));
                let removed_list = world.new_allocated(Some(removed));
                Ok(ScriptValue::List(vec![added_list, removed_list]))
            } else {
                Ok(ScriptValue::Unit)
            }
        } else {
            Ok(ScriptValue::Unit)
        }
    })?
}
fn script_has_effect(
    ctx: FunctionCallContext,
    script_value: ScriptValue,
) -> Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let (entity, skill_id) = match script_value {
        // it must be a list of two items: entity and skill_id
        ScriptValue::List(list) if list.len() == 2 => {
            let entity = list
                .first()
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
            let skill_id = list
                .get(1)
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

            if let (ScriptValue::Reference(entity_ref), ScriptValue::Reference(skill_id_ref)) =
                (entity, skill_id)
            {
                let entity = entity_ref.downcast::<Entity>(world_guard.clone())?;
                let skill_id_ref = skill_id_ref.downcast::<skills::Id>(world_guard.clone())?;
                (entity, skill_id_ref)
            } else {
                return Err(InteropError::type_mismatch(
                    std::any::TypeId::of::<Entity>(),
                    None,
                ));
            }
        }
        _ => return Err(InteropError::argument_count_mismatch(2, 1)),
    };

    world_guard.with_global_access(|world| {
        if let Ok(entity_world_ref) = world.get_entity(entity) {
            if let Some(abnormal_effects) = entity_world_ref.get::<AbnormalEffects>() {
                let has_effect = abnormal_effects.has_effect(skill_id);
                ScriptValue::Bool(has_effect)
            } else {
                ScriptValue::Bool(false) // Entity doesn't have AbnormalEffects component
            }
        } else {
            ScriptValue::Bool(false) // Entity not found
        }
    })
}

fn script_remove(
    ctx: FunctionCallContext,
    script_value: ScriptValue,
) -> Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let (entity, skill_id) = match script_value {
        // it must be a list of two items: entity and skill_id
        ScriptValue::List(list) if list.len() == 2 => {
            let entity = list
                .first()
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
            let skill_id = list
                .get(1)
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

            if let (ScriptValue::Reference(entity_ref), ScriptValue::Reference(skill_id_ref)) =
                (entity, skill_id)
            {
                let entity = entity_ref.downcast::<Entity>(world_guard.clone())?;
                let skill_id_ref = skill_id_ref.downcast::<skills::Id>(world_guard.clone())?;

                (entity, skill_id_ref)
            } else {
                return Err(InteropError::type_mismatch(
                    std::any::TypeId::of::<Entity>(),
                    None,
                ));
            }
        }
        _ => return Err(InteropError::argument_count_mismatch(2, 1)),
    };

    world_guard.with_global_access(|world| {
        if let Ok(mut entity_world_mut) = world.get_entity_mut(entity) {
            let removed =
                if let Some(mut abnormal_effects) = entity_world_mut.get_mut::<AbnormalEffects>() {
                    abnormal_effects.remove_by_skill_id(skill_id)
                } else {
                    false
                };

            if removed {
                // Also remove timer if present
                if let Some(mut timers) = entity_world_mut.get_mut::<AbnormalEffectsTimers>() {
                    timers.remove(skill_id);
                }
                ScriptValue::Integer(1) // Successfully removed
            } else {
                ScriptValue::Integer(0) // Effect not found or no component
            }
        } else {
            ScriptValue::Integer(0) // Entity not found
        }
    })
}
