use bevy::prelude::*;
use scripting::prelude::NamespaceBuilder;

mod can_move_to;
mod can_see_target;
mod request_visibility_check;

// Wrapper struct for RegionGeoData operations namespace
#[derive(Clone, Debug, Reflect)]
pub struct RegionGeoData;

pub struct RegionGeoDataScriptingPlugin;

impl Plugin for RegionGeoDataScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Register RegionGeoData operations
        let world = app.world_mut();
        NamespaceBuilder::<RegionGeoData>::new(world)
            .register("can_move_to", can_move_to::script_can_move_to)
            .register("can_see_target", can_see_target::script_can_see_target)
            .register(
                "request_visibility_check",
                request_visibility_check::script_request_visibility_check,
            );
    }
}
