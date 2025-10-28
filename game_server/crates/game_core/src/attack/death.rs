use crate::{
    action::pickup::PickupRequest,
    active_action::ActiveAction,
    attack::Attacking,
    movement::{Following, Movement},
    npc::DialogRequest,
    player_specific::next_intention::NextIntention,
};
use bevy::prelude::*;
use bevy_ecs::{
    component::{ComponentHook, Immutable, StorageType},
    relationship::RelationshipHookMode,
    world::DeferredWorld,
};
use std::time::Duration;

pub struct DeathComponentsPlugin;
impl Plugin for DeathComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Dead>()
            .register_type::<FakeDead>()
            .register_type::<DeadTimer>();
    }
}

#[derive(Component, Debug, Reflect)]
pub struct FakeDead;

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct Dead {
    killer: Entity,
}

impl Component for Dead {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Immutable;

    fn on_insert() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            match ctx.relationship_hook_mode {
                RelationshipHookMode::Run => {}
                _ => return,
            }

            world.commands().entity(ctx.entity).remove::<(
                Movement,
                Following,
                Attacking,
                ActiveAction,
                NextIntention,
                PickupRequest,
                DialogRequest,
            )>();
        })
    }
}

impl Dead {
    pub fn new(killer: Entity) -> Self {
        Self { killer }
    }
    pub fn killer(&self) -> Entity {
        self.killer
    }
}

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct DeadTimer(Timer);
impl Default for DeadTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(7), TimerMode::Once))
    }
}
