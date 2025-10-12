use crate::{
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{AttackStanceStart, AttackStanceStop},
    },
    object_id::ObjectId,
};
use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, Mutable, StorageType},
        world::DeferredWorld,
    },
    log,
    prelude::*,
};
use std::time::Duration;

#[derive(Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct InCombat(Timer);

impl Component for InCombat {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Mutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            let Some(object_id) = world.entity(context.entity).get::<ObjectId>().copied() else {
                log::error!(
                    "InCombat on_add: No ObjectId found for entity {:?}",
                    context.entity
                );
                return;
            };
            world.commands().trigger_targets(
                ServerPacketBroadcast::new(AttackStanceStart::new(object_id).into()),
                context.entity,
            );
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            let Some(object_id) = world.entity(context.entity).get::<ObjectId>().copied() else {
                log::error!(
                    "InCombat on_remove: No ObjectId found for entity {:?}",
                    context.entity
                );
                return;
            };
            world.commands().trigger_targets(
                ServerPacketBroadcast::new(AttackStanceStop::new(object_id).into()),
                context.entity,
            );
        })
    }
}

impl Default for InCombat {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(10), TimerMode::Once))
    }
}
