use bevy::prelude::*;

mod loading;
mod mechanics;
mod system;

use bevy::ecs::schedule::ScheduleLabel;
pub use loading::*;
pub use mechanics::*;
pub use system::*;

pub struct GameServerStatePlugin;
impl Plugin for GameServerStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameServerStateSystems>();

        configure_game_server_state_sets(app, Update);
        configure_game_server_state_sets(app, FixedUpdate);

        app.add_plugins((GameSystemPlugin, GameMechanicsPlugin));
    }
}

fn configure_game_server_state_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            GameServerStateSystems::Load.run_if(in_state(GameServerStateSystems::Load)),
            GameServerStateSystems::Run.run_if(in_state(GameServerStateSystems::Run)),
            GameServerStateSystems::Pause.run_if(
                in_state(GameServerStateSystems::Pause)
                    .or(in_state(GameServerStateSystems::ForcePause)),
            ),
            GameServerStateSystems::Shutdown.run_if(in_state(GameServerStateSystems::Shutdown)),
        ),
    );
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States, SystemSet)]
pub enum GameServerStateSystems {
    #[default]
    Load,
    Run,
    Pause,
    ForcePause,
    Shutdown,
}

#[derive(Event)]
pub struct TogglePause;
