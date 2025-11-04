use bevy_ecs::system::SystemState;
use map::WorldMapQuery;
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::ScriptValue,
    utils::extract_positions,
};

/// Scripting function to check if line of sight exists between start and target position
///
/// Expected input: A table with two Vec3 fields:
/// - start: Starting position (Vec3)
/// - target: Target position (Vec3)
///
/// Returns: boolean - true if target is visible from start, false otherwise
pub(crate) fn script_can_see_target(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;
    let (start, target) = extract_positions(&data, world_guard.clone())?;
    world_guard.with_global_access(|world| {
        let mut map_state: SystemState<WorldMapQuery> = SystemState::new(world);
        let map_query = map_state.get_mut(world);

        ScriptValue::Bool(map_query.can_see_target(start, target))
    })
}
