use super::system::GameServerSystems;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use strum::EnumIter;

mod items;
mod stats;

pub use items::*;
pub use stats::*;

pub struct GameMechanicsPlugin;

impl Plugin for GameMechanicsPlugin {
    fn build(&self, app: &mut App) {
        configure_game_mechanics_sets(app, Update);
        configure_game_mechanics_sets(app, FixedUpdate);

        app.add_plugins((ItemMechanicsPlugin, StatKindPlugin));
    }
}

fn configure_game_mechanics_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            GameMechanicsSystems::Skills.in_set(GameServerSystems::Mechanics),
            GameMechanicsSystems::AbnormalUpdates
                .in_set(GameServerSystems::Mechanics)
                .after(GameMechanicsSystems::Skills),
            GameMechanicsSystems::StatsCalculation
                .in_set(GameServerSystems::Mechanics)
                .after(GameMechanicsSystems::AbnormalUpdates),
            GameMechanicsSystems::Attacking.in_set(GameServerSystems::Mechanics),
            GameMechanicsSystems::Items
                .after(GameMechanicsSystems::Attacking)
                .in_set(GameServerSystems::Mechanics),
            GameMechanicsSystems::NextIntention
                .after(GameMechanicsSystems::Attacking)
                .after(GameMechanicsSystems::Skills)
                .after(GameMechanicsSystems::Items)
                .in_set(GameServerSystems::Mechanics),
        ),
    );
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq, SystemSet)]
pub enum GameMechanicsSystems {
    Items,
    Skills,
    AbnormalUpdates,
    StatsCalculation,
    Attacking,
    NextIntention,
}
