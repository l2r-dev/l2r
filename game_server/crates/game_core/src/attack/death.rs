use bevy::prelude::*;
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

#[derive(Clone, Component, Copy, Debug, Event, Reflect)]
pub struct Dead {
    killer: Entity,
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
