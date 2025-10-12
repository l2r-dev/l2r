use crate::animation::Animation;
use bevy::prelude::*;
use strum::Display;

const PICKUP_ANIMATION_DURATION: f32 = 0.2;

pub struct PickupComponentsPlugin;
impl Plugin for PickupComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PickupRequest>()
            .register_type::<PickupAnimation>();
    }
}

#[derive(Clone, Component, Copy, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PickupRequest(pub Entity);

#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
#[require(Animation)]
pub struct PickupAnimation(Timer, Entity);

impl PickupAnimation {
    pub fn new(entity: Entity) -> Self {
        Self(
            Timer::from_seconds(PICKUP_ANIMATION_DURATION, TimerMode::Once),
            entity,
        )
    }

    pub fn timer(&self) -> &Timer {
        &self.0
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.0
    }

    pub fn entity(&self) -> Entity {
        self.1
    }
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PickupMetric {
    ItemsPickedUp,
}
