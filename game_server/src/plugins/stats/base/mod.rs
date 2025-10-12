use bevy::prelude::*;

mod attack;
mod critical;
mod defence;
mod movement;
mod other;
mod primal;
mod progress;
mod vitals;

pub struct BaseStatsPlugin;
impl Plugin for BaseStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(primal::PrimalStatsPlugin)
            .add_plugins(attack::AttackStatsPlugin)
            .add_plugins(critical::CriticalStatsPlugin)
            .add_plugins(defence::DefenceStatsPlugin)
            .add_plugins(vitals::VitalsStatsPlugin)
            .add_plugins(other::OtherStatsPlugin)
            .add_plugins(progress::ProgressStatsPlugin)
            .add_plugins(movement::MovementStatsPlugin);
    }
}
