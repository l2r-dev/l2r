use super::GameServerStateSystems;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub struct GameSystemPlugin;

impl Plugin for GameSystemPlugin {
    fn build(&self, app: &mut App) {
        configure_game_system_sets(app, Update);
        configure_game_system_sets(app, FixedUpdate);
    }
}

fn configure_game_system_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            GameServerSystems::Database.before(GameServerSystems::Mechanics),
            GameServerSystems::Mechanics.in_set(GameServerStateSystems::Run),
        ),
    );
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum GameServerSystems {
    Database,
    Mechanics,
}
