use bevy::{platform::collections::HashMap, prelude::*};
use game_core::chat::Kind;
use std::time::Duration;

const CHAT_COOLDOWN_SECONDS: f32 = 0.2;

pub struct ChatFloodProtectionPlugin;
impl Plugin for ChatFloodProtectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChatCooldown>()
            .add_systems(Update, tick_chat_cooldowns);
    }
}

fn tick_chat_cooldowns(time: Res<Time>, mut query: Query<&mut ChatCooldown>) {
    let delta = time.delta();
    query.par_iter_mut().for_each(|mut cooldown| {
        cooldown.tick(delta);
    });
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ChatCooldown {
    timers: HashMap<Kind, Timer>,
}

impl Default for ChatCooldown {
    fn default() -> Self {
        Self {
            timers: HashMap::with_capacity(4),
        }
    }
}

impl ChatCooldown {
    pub fn can_send(&self, kind: &Kind) -> bool {
        self.timers
            .get(kind)
            .map(|timer| timer.finished())
            .unwrap_or(true)
    }

    pub fn start_cooldown(&mut self, kind: Kind) {
        let timer = Timer::from_seconds(CHAT_COOLDOWN_SECONDS, TimerMode::Once);
        self.timers.insert(kind, timer);
    }

    pub fn remaining(&self, kind: &Kind) -> Duration {
        self.timers
            .get(kind)
            .map(|timer| {
                if timer.finished() {
                    Duration::ZERO
                } else {
                    Duration::from_secs_f32(timer.remaining_secs())
                }
            })
            .unwrap_or(Duration::ZERO)
    }

    fn tick(&mut self, delta: Duration) {
        for timer in self.timers.values_mut() {
            timer.tick(delta);
        }
    }
}
