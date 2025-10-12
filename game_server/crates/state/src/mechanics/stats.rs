use super::GameMechanicsSystems;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use strum::EnumIter;

pub struct StatKindPlugin;

impl Plugin for StatKindPlugin {
    fn build(&self, app: &mut App) {
        configure_stat_kind_sets(app, Update);
        configure_stat_kind_sets(app, FixedUpdate);
    }
}

fn configure_stat_kind_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            // Primal stats must be calculated first
            StatKindSystems::Primal.in_set(GameMechanicsSystems::StatsCalculation),
            // All other stats run after Primal
            StatKindSystems::Vitals
                .after(StatKindSystems::Primal)
                .after(StatKindSystems::Progress)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Attack
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Defence
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Movement
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Critical
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::ElementPower
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Inventory
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::MpConsumption
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Progress
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::ProgressLevel
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::ProgressRates
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
            StatKindSystems::Other
                .after(StatKindSystems::Primal)
                .in_set(GameMechanicsSystems::StatsCalculation),
        ),
    );
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq, SystemSet)]
pub enum StatKindSystems {
    Vitals,
    Attack,
    Defence,
    Movement,
    Critical,
    Primal,
    ElementPower,
    Inventory,
    MpConsumption,
    Progress,
    ProgressLevel,
    ProgressRates,
    Other,
}
