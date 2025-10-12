use crate::skills;
use bevy::{platform::collections::HashMap, prelude::*, time::Timer};
use core::fmt;

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
pub struct SkillReuseTimers(HashMap<skills::Id, Timer>);

impl fmt::Debug for SkillReuseTimers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkillReuseTimers({})", self.0.len())
    }
}

impl SkillReuseTimers {
    /// Starts a reuse timer for a skill
    pub fn start_reuse(&mut self, skill_id: skills::Id, reuse_delay_ms: u32) {
        let duration = std::time::Duration::from_millis(reuse_delay_ms as u64);
        let mut timer = Timer::new(duration, TimerMode::Once);
        timer.tick(std::time::Duration::ZERO); // Start the timer
        self.0.insert(skill_id, timer);
    }

    /// Checks if a skill is on cooldown (reuse timer active)
    pub fn is_skill_on_cooldown(&self, skill_id: skills::Id) -> bool {
        self.0
            .get(&skill_id)
            .map(|timer| !timer.finished())
            .unwrap_or(false)
    }

    /// Gets the remaining cooldown time in milliseconds, returns 0 if not on cooldown
    pub fn remaining_cooldown_ms(&self, skill_id: skills::Id) -> u64 {
        self.0
            .get(&skill_id)
            .filter(|timer| !timer.finished())
            .map(|timer| timer.remaining().as_millis() as u64)
            .unwrap_or(0)
    }

    /// Ticks all active reuse timers
    pub fn tick(&mut self, delta: std::time::Duration) {
        // Tick all timers and collect finished ones
        let finished: Vec<skills::Id> = self
            .0
            .iter_mut()
            .filter_map(|(id, timer)| {
                timer.tick(delta);
                if timer.finished() { Some(*id) } else { None }
            })
            .collect();

        // Remove finished timers
        for id in finished {
            self.0.remove(&id);
        }
    }

    /// Clears all reuse timers (useful for admin commands or special effects)
    pub fn clear_all(&mut self) {
        self.0.clear();
    }

    /// Clears a specific skill's reuse timer
    pub fn clear_skill(&mut self, skill_id: skills::Id) {
        self.0.remove(&skill_id);
    }
}
