use super::SendStopMove;
use crate::{
    action::wait_kind::Sit,
    attack::Dead,
    network::packets::server::{ActionFail, GameServerPacket},
    path_finding::WAYPOINTS_CAPACITY,
};
use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, Immutable, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_ecs::{component::Mutable, relationship::RelationshipHookMode};
use smallvec::SmallVec;
use spatial::WayPoint;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct MoveToEntity {
    pub target: Entity,
    pub range: f32,
}

impl Component for MoveToEntity {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Immutable;

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
            world
                .commands()
                .trigger_targets(GameServerPacket::from(ActionFail), context.entity);
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct MoveTarget {
    waypoints: VecDeque<WayPoint>,
}

impl Component for MoveTarget {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
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
            world
                .commands()
                .trigger_targets(GameServerPacket::from(ActionFail), context.entity);
        })
    }
}

impl MoveTarget {
    pub fn new(waypoints: VecDeque<WayPoint>) -> Self {
        Self { waypoints }
    }

    pub fn single(waypoint: WayPoint) -> Self {
        Self {
            waypoints: VecDeque::from([waypoint]),
        }
    }
}

impl std::ops::Deref for MoveTarget {
    type Target = VecDeque<WayPoint>;

    fn deref(&self) -> &Self::Target {
        &self.waypoints
    }
}

impl std::ops::DerefMut for MoveTarget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.waypoints
    }
}

impl From<Vec<WayPoint>> for MoveTarget {
    fn from(waypoints: Vec<WayPoint>) -> Self {
        Self {
            waypoints: VecDeque::from(waypoints),
        }
    }
}

impl From<SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>> for MoveTarget {
    fn from(waypoints: SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>) -> Self {
        let mut mt = VecDeque::with_capacity(waypoints.len());
        mt.extend(waypoints);
        Self { waypoints: mt }
    }
}
