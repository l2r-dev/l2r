use bevy::prelude::*;
use smallvec::SmallVec;
use spatial::WayPoint;
use std::time::Duration;

pub struct PathFindingComponentsPlugin;
impl Plugin for PathFindingComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InActionPathfindingTimer>();

        app.add_event::<DirectMoveRequest>()
            .add_event::<DirectMoveResult>()
            .add_event::<PathFindingRequest>()
            .add_event::<PathFindingResult>()
            .add_event::<DropDebugItem>();
    }
}

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct DirectMoveRequest {
    pub entity: Entity,
    pub start: Vec3,
    pub target: Vec3,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct DirectMoveResult {
    pub start: Vec3,
    pub target: Vec3,
    pub can_move: bool,
}

#[derive(Clone, Copy, Debug, Event, PartialEq)]
pub struct PathFindingRequest {
    pub start: Vec3,
    pub goal: Vec3,
    pub max_iterations: usize,
}

pub const WAYPOINTS_CAPACITY: usize = 16;

#[derive(Clone, Debug, Event, PartialEq)]
pub struct PathFindingResult {
    pub path: SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>,
    pub start: Vec3,
    pub goal: Vec3,
}

const PATHFINDING_COOLDOWN: Duration = Duration::from_millis(1000);

#[derive(Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct InActionPathfindingTimer(Timer);
impl Default for InActionPathfindingTimer {
    fn default() -> Self {
        InActionPathfindingTimer(Timer::new(PATHFINDING_COOLDOWN, TimerMode::Once))
    }
}

#[derive(Event)]
pub struct DropDebugItem(pub Vec3);
