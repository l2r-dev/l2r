use bevy::{
    app::{App, AppExit},
    ecs::error::GLOBAL_ERROR_HANDLER,
};
use l2r_loginserver::plugins::Core;

fn main() -> AppExit {
    GLOBAL_ERROR_HANDLER
        .set(bevy::ecs::error::warn)
        .expect("The error handler can only be set once.");

    App::new().add_plugins(Core).run()
}
