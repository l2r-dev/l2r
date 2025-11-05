use bevy::prelude::*;
use game_core::path_finding::DirectMoveRequest;
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::ScriptValue,
};
use std::any::TypeId;

/// Sends a DirectMoveRequest event that will be processed by the pathfinding system
pub(crate) fn script_request_visibility_check(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let (entity, start, target) = match data {
        ScriptValue::Map(ref table) => {
            let entity_value = table.get("entity").ok_or_else(|| {
                InteropError::string(
                    "Missing required field 'entity' in visibility check request".to_string(),
                )
            })?;
            let start_value = table.get("start").ok_or_else(|| {
                InteropError::string(
                    "Missing required field 'start' in visibility check request".to_string(),
                )
            })?;
            let target_value = table.get("target").ok_or_else(|| {
                InteropError::string(
                    "Missing required field 'target' in visibility check request".to_string(),
                )
            })?;

            let entity = match entity_value {
                ScriptValue::Reference(entity_ref) => {
                    entity_ref.downcast::<Entity>(world_guard.clone())?
                }
                _ => {
                    return Err(InteropError::value_mismatch(
                        TypeId::of::<Entity>(),
                        entity_value.clone(),
                    )
                    .with_context("'entity' field must be an Entity reference"));
                }
            };

            let start = match start_value {
                ScriptValue::Reference(vec_ref) => vec_ref.downcast::<Vec3>(world_guard.clone())?,
                _ => {
                    return Err(InteropError::value_mismatch(
                        TypeId::of::<Vec3>(),
                        start_value.clone(),
                    )
                    .with_context("'start' field must be a Vec3 reference"));
                }
            };

            let target = match target_value {
                ScriptValue::Reference(vec_ref) => vec_ref.downcast::<Vec3>(world_guard.clone())?,
                _ => {
                    return Err(InteropError::value_mismatch(
                        TypeId::of::<Vec3>(),
                        target_value.clone(),
                    )
                    .with_context("'target' field must be a Vec3 reference"));
                }
            };

            (entity, start, target)
        }
        _ => {
            return Err(InteropError::string_type_mismatch(
                "Map with fields: entity (Entity), start (Vec3), target (Vec3)".to_string(),
                None,
            ));
        }
    };

    world_guard.with_global_access(|world| {
        world.send_event(DirectMoveRequest {
            entity,
            start,
            target,
        });
    })?;

    Ok(ScriptValue::Unit)
}
