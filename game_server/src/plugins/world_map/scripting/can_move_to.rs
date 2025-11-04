use bevy_ecs::system::SystemState;
use map::WorldMapQuery;
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::ScriptValue,
    utils::extract_positions,
};

/// Scripting function to check if movement is possible from start to goal position
///
/// Expected input: A table with two Vec3 fields:
/// - start: Starting position (Vec3)
/// - goal: Goal position (Vec3)
///
/// Returns: boolean - true if movement is possible, false otherwise
pub(crate) fn script_can_move_to(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;
    let (start, goal) = extract_positions(&data, world_guard.clone())?;
    world_guard.with_global_access(|world| {
        let mut map_state: SystemState<WorldMapQuery> = SystemState::new(world);
        let map_query = map_state.get_mut(world);
        ScriptValue::Bool(map_query.can_move_to(start, goal))
    })
}
