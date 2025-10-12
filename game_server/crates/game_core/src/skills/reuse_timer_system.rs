use crate::skills::SkillReuseTimers;
use bevy::prelude::*;

pub struct SkillReuseTimerPlugin;

impl Plugin for SkillReuseTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_skill_reuse_timers);
    }
}

/// System that ticks all skill reuse timers
fn tick_skill_reuse_timers(time: Res<Time>, mut query: Query<&mut SkillReuseTimers>) {
    let delta = time.delta();

    query.par_iter_mut().for_each(|mut reuse_timers| {
        reuse_timers.tick(delta);
    });
}
