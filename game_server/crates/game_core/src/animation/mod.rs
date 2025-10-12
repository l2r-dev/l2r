use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, Immutable, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use std::time::Duration;

pub struct AnimationComponentPlugin;
impl Plugin for AnimationComponentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Animation>()
            .register_type::<AnimationTimer>();
    }
}

/// A marker component indicating that an entity is currently performing an action.
///
/// This component is used as a flag to prevent entities from starting new actions
/// (such as movement, attacking, or picking up items) while they are still executing
/// a previous action. It acts as a synchronization mechanism to ensure entities
/// complete their current animation or action before beginning another.
/// - [`AnimationTimer`]: Automatically removes this component when the timer expires
/// - [`AnimationFinished`]: Event triggered when this component is removed
#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Animation;
impl Component for Animation {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Immutable;

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            world
                .commands()
                .trigger_targets(AnimationFinished, context.entity);
        })
    }
}

#[derive(Clone, Debug, Default, Event, Reflect)]
pub struct AnimationFinished;

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
#[require(Animation)]
pub struct AnimationTimer(Timer);
impl AnimationTimer {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}
