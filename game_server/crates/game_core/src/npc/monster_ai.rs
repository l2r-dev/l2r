use crate::{attack::InCombat, movement::MoveTarget};
use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct NpcAiComponentsPlugin;
impl Plugin for NpcAiComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RandomWalkingTimer>();
    }
}

#[derive(Clone, Component, Copy, Debug, Default, Deserialize, Serialize)]
pub struct MonsterAiParams {
    pub aggro_range: Option<u32>,
    pub clan_help_range: Option<u32>,
    #[serde(default)]
    pub is_aggressive: bool,
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct RandomWalkingTimer(Timer);
impl RandomWalkingTimer {
    pub const RANDOM_RANGE: std::ops::Range<f32> = 1.0..90.0;
    pub fn random_duration() -> Duration {
        Duration::from_secs_f32(rand::thread_rng().gen_range(Self::RANDOM_RANGE))
    }
}
impl Default for RandomWalkingTimer {
    fn default() -> Self {
        Self(Timer::new(Self::random_duration(), TimerMode::Repeating))
    }
}

#[derive(QueryFilter)]
struct WalkingSetupFilter {
    monster: With<super::kind::Monster>,
    no_timer: Without<RandomWalkingTimer>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RandomWalkingQuery<'a> {
    entity: Entity,
    transform: &'a Transform,
    timer: &'a mut RandomWalkingTimer,
}

#[derive(QueryFilter)]
struct RandomWalkingFilter {
    move_target: Without<MoveTarget>,
    not_in_combat: Without<InCombat>,
}
