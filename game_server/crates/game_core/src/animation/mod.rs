use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_ecs::component::Mutable;
use std::time::Duration;

pub struct AnimationComponentPlugin;
impl Plugin for AnimationComponentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Animation>();
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
#[derive(Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Animation {
    timer: Timer,
}

impl Animation {
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    ///Returns true if finished
    pub fn proceed_timer(&mut self, dt: Duration) -> bool {
        self.timer.tick(dt);

        self.timer.finished()
    }
}

impl Component for Animation {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Mutable;

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
