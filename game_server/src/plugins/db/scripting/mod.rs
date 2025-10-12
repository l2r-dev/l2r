use bevy::prelude::*;
use scripting::prelude::NamespaceBuilder;

mod create_or_update;
mod find_by_id;
mod query_raw;

// Wrapper struct for database operations namespace
#[derive(Clone, Debug, Reflect)]
pub struct DatabaseOps;

pub struct DatabaseScriptingPlugin;

impl Plugin for DatabaseScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Register core database operations
        let world = app.world_mut();
        NamespaceBuilder::<DatabaseOps>::new(world)
            .register("find_by_id", find_by_id::script_find_by_id)
            .register(
                "create_or_update",
                create_or_update::script_create_or_update,
            )
            .register("query_raw", query_raw::script_query_raw);
        // .register("find_by_conditions", script_find_by_conditions)
        // .register("update", script_update)
        // .register("delete", script_delete)
        // .register("execute_custom", script_execute_custom);
    }
}
