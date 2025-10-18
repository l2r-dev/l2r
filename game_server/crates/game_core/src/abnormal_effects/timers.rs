use crate::{
    skills::Id,
    stats::{StatKind, StatsOperation},
};
use bevy::{platform::collections::HashMap, prelude::*};
use core::fmt;
use std::time::Duration;

#[derive(Clone, Debug, Reflect)]
pub struct AbnormalEffectTimer {
    timer: Option<Timer>,
    effects_over_time: Option<Vec<EffectOverTime>>,
}

impl AbnormalEffectTimer {
    pub fn new(timer: Option<Timer>, effects_over_time: Option<Vec<EffectOverTime>>) -> Self {
        Self {
            timer,
            effects_over_time,
        }
    }

    pub fn timer(&self) -> Option<&Timer> {
        self.timer.as_ref()
    }

    pub fn timer_mut(&mut self) -> Option<&mut Timer> {
        self.timer.as_mut()
    }

    pub fn effects_over_time(&self) -> Option<&Vec<EffectOverTime>> {
        self.effects_over_time.as_ref()
    }

    pub fn effects_over_time_mut(&mut self) -> Option<&mut Vec<EffectOverTime>> {
        self.effects_over_time.as_mut()
    }
}

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
pub struct AbnormalEffectsTimers(HashMap<Id, AbnormalEffectTimer>);

impl fmt::Debug for AbnormalEffectsTimers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AbnormalEffectsTimers({})", self.0.len())
    }
}

impl AbnormalEffectsTimers {
    pub fn insert(&mut self, skill_id: Id, timer: AbnormalEffectTimer) {
        self.0.insert(skill_id, timer);
    }

    pub fn remove(&mut self, skill_id: Id) -> Option<AbnormalEffectTimer> {
        self.0.remove(&skill_id)
    }

    pub fn get_timer(&self, skill_id: Id) -> Option<&AbnormalEffectTimer> {
        self.0.get(&skill_id)
    }

    pub fn get_timer_mut(&mut self, skill_id: Id) -> Option<&mut AbnormalEffectTimer> {
        self.0.get_mut(&skill_id)
    }

    pub fn effects_over_time_mut(&mut self) -> impl Iterator<Item = (Id, &mut EffectOverTime)> {
        self.0
            .iter_mut()
            .filter_map(|(skill_id, timer)| {
                timer.effects_over_time.as_mut().map(|vec| (*skill_id, vec))
            })
            .flat_map(|(skill_id, vec)| vec.iter_mut().map(move |effect| (skill_id, effect)))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct EffectOverTime {
    timer: Timer,
    stat: StatKind,
    operation: StatsOperation<f32>,
}

impl EffectOverTime {
    pub fn new(duration: Duration, stat: StatKind, operation: StatsOperation<f32>) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            stat,
            operation,
        }
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn operation(&self) -> &StatsOperation<f32> {
        &self.operation
    }

    pub fn stat_kind(&self) -> StatKind {
        self.stat
    }
}

impl AsRef<StatKind> for EffectOverTime {
    fn as_ref(&self) -> &StatKind {
        &self.stat
    }
}

impl AsRef<StatsOperation<f32>> for EffectOverTime {
    fn as_ref(&self) -> &StatsOperation<f32> {
        &self.operation
    }
}
