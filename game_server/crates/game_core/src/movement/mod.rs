use crate::{
    action::wait_kind::Sit, attack::Dead, path_finding::WAYPOINTS_CAPACITY, stats::Movable,
};
use bevy::prelude::*;
use bevy_ecs::{
    component::{ComponentHook, HookContext, Mutable, StorageType},
    relationship::RelationshipHookMode,
    world::DeferredWorld,
};
use smallvec::SmallVec;
use spatial::WayPoint;
use std::collections::VecDeque;

mod follow;
mod move_mode;

pub use follow::*;
pub use move_mode::*;

#[derive(Event)]
pub struct ArrivedAtWaypoint;

#[derive(Event)]
pub struct MoveStep;

pub struct MovementComponentsPlugin;
impl Plugin for MovementComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Movable>()
            .register_type::<Movement>()
            .register_type::<Falling>();
    }
}

#[derive(Clone, Copy, Event, Reflect)]
pub struct SendStopMove;

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct LookAt(pub Entity);

#[derive(Clone, Component, Copy, Debug, Default, Reflect)]
pub struct Falling;

#[derive(Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub enum Movement {
    ToLocation { waypoints: VecDeque<WayPoint> },
    ToEntity { target: Entity, range: f32 },
}

impl Component for Movement {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            match ctx.relationship_hook_mode {
                RelationshipHookMode::Run => {}
                _ => return,
            }

            let entity_ref = world.entity(ctx.entity);

            if entity_ref.get::<Sit>().is_some() || entity_ref.get::<Dead>().is_some() {
                world.commands().entity(ctx.entity).remove::<Self>();
            }
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            world
                .commands()
                .trigger_targets(SendStopMove, context.entity);
        })
    }
}

impl Movement {
    pub fn to_entity(target: Entity, range: f32) -> Self {
        Self::ToEntity { target, range }
    }

    pub fn to_location(waypoints: VecDeque<WayPoint>) -> Self {
        Self::ToLocation { waypoints }
    }

    pub fn to_waypoint(waypoint: WayPoint) -> Self {
        Self::ToLocation {
            waypoints: VecDeque::from([waypoint]),
        }
    }

    pub fn waypoints(&self) -> Option<&VecDeque<WayPoint>> {
        match self {
            Self::ToLocation { waypoints } => Some(waypoints),
            _ => None,
        }
    }

    pub fn waypoints_mut(&mut self) -> Option<&mut VecDeque<WayPoint>> {
        match self {
            Self::ToLocation { waypoints } => Some(waypoints),
            _ => None,
        }
    }

    pub fn target(&self) -> Option<Entity> {
        match self {
            Self::ToEntity { target, .. } => Some(*target),
            _ => None,
        }
    }

    pub fn range(&self) -> Option<f32> {
        match self {
            Self::ToEntity { range, .. } => Some(*range),
            _ => None,
        }
    }

    pub fn is_to_location(&self) -> bool {
        matches!(self, Self::ToLocation { .. })
    }

    pub fn is_to_entity(&self) -> bool {
        matches!(self, Self::ToEntity { .. })
    }
}

impl From<Vec<WayPoint>> for Movement {
    fn from(waypoints: Vec<WayPoint>) -> Self {
        Self::ToLocation {
            waypoints: VecDeque::from(waypoints),
        }
    }
}

impl From<SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>> for Movement {
    fn from(waypoints: SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>) -> Self {
        let mut wps = VecDeque::with_capacity(waypoints.len());
        wps.extend(waypoints);
        Self::ToLocation { waypoints: wps }
    }
}
