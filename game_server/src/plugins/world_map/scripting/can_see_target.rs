use bevy::prelude::*;
use map::{RegionGeoData, WorldMap};
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

    // Extract start and target from the data
    let (start_vec3, target_vec3) = extract_positions(&data, world_guard.clone())?;

    // Convert Vec3 to GeoVec3
    let start = WorldMap::vec3_to_geo(start_vec3);
    let target = WorldMap::vec3_to_geo(target_vec3);

    // Get the region geodata
    world_guard.with_global_access(|world| {
        let world_map = world.resource::<map::WorldMap>();
        let regions_geodata = world.resource::<Assets<RegionGeoData>>();

        // Get geodata for the start position's region
        let region_id = map::id::RegionId::from(start);

        // Get the region entity, component, and geodata
        if let Some(region_entity) = world_map.get(&region_id)
            && let Some(region) = world.get::<map::Region>(*region_entity)
            && let Some(geodata) = regions_geodata.get(region.handle().id())
        {
            // Check line of sight
            let can_see = geodata.can_see_target(start, target);
            return ScriptValue::Bool(can_see);
        }

        // If any lookup failed, return false
        ScriptValue::Bool(false)
    })
}
