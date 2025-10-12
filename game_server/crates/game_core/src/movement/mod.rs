use super::{action::wait_kind::Sit, animation::Animation, attack::Dead};
use crate::stats::Movable;
use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};

mod follow;
mod move_mode;
mod move_target;

pub use follow::*;
pub use move_mode::*;
pub use move_target::*;

#[derive(Event)]
pub struct ArrivedAtWaypoint;

#[derive(Event)]
pub struct MoveStep;

pub struct MovementComponentsPlugin;
impl Plugin for MovementComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Movable>()
            .register_type::<MoveTarget>()
            .register_type::<MoveToEntity>();
    }
}

#[derive(Clone, Copy, Event, Reflect)]
pub struct SendStopMove;

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct LookAt(pub Entity);

#[derive(QueryFilter)]
struct MoveFilter {
    not_animating: Without<Animation>,
    not_dead: Without<Dead>,
    not_sitting: Without<Sit>,
}

#[derive(QueryData)]
struct MoveQuery<'a> {
    entity: Entity,
    transform: Ref<'a, Transform>,
    move_target: Option<Ref<'a, MoveTarget>>,
    move_to_entity: Option<Ref<'a, MoveToEntity>>,
}
