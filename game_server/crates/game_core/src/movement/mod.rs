use crate::stats::Movable;
use bevy::prelude::*;

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
