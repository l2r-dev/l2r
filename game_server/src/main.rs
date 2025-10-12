use bevy::{ecs::error::GLOBAL_ERROR_HANDLER, prelude::*};
use l2r_gameserver::plugins;

fn main() -> AppExit {
    GLOBAL_ERROR_HANDLER
        .set(bevy::ecs::error::warn)
        .expect("The error handler can only be set once.");

    App::new()
        .add_plugins(plugins::CoreWithInfrastructure)
        .run()
}
