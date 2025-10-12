use bevy::prelude::*;
use game_core::abnormal_effects::{
    AbnormalEffects, AbnormalEffectsChangeTracker, AbnormalEffectsComponentsPlugin,
    AbnormalEffectsTimers,
};
use state::GameMechanicsSystems;
use std::time::Duration;

mod request_dispel;

mod effects_scripting;
mod over_time;

pub struct AbnormalEffectsPlugin;

impl Plugin for AbnormalEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbnormalEffectsComponentsPlugin)
            .add_plugins(request_dispel::RequestDispelPlugin);

        app.add_systems(
            Update,
            handle_abnormal_effect_timers.in_set(GameMechanicsSystems::AbnormalUpdates),
        )
        .add_systems(
            Update,
            over_time::handle_abnormal_effect_effects_over_time
                .after(handle_abnormal_effect_timers),
        )
        .add_systems(Update, track_abnormal_effects_changes);

        effects_scripting::register_script_functions(app);
    }
}

fn handle_abnormal_effect_timers(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut query: Query<(Mut<AbnormalEffects>, Mut<AbnormalEffectsTimers>)>,
) {
    let time_spent = time.elapsed_secs() - *last_time;
    if time_spent >= 0.5 {
        *last_time = time.elapsed_secs();

        for (mut abnormal_effects, mut timers) in query.iter_mut() {
            let mut finished_skills = Vec::new();

            // Tick all timers and collect finished ones
            for (skill_id, timer_data) in timers.iter_mut() {
                if let Some(timer) = timer_data.timer_mut() {
                    timer.tick(Duration::from_secs_f32(time_spent));
                    if timer.finished() {
                        finished_skills.push(*skill_id);
                    }
                }
            }

            // Remove finished effects and their timers
            for skill_id in finished_skills {
                abnormal_effects.remove(skill_id);
                timers.remove(skill_id);
            }
        }
    }
}

fn track_abnormal_effects_changes(
    mut query: Query<
        (Ref<AbnormalEffects>, Mut<AbnormalEffectsChangeTracker>),
        Changed<AbnormalEffects>,
    >,
) {
    for (current, mut tracker) in &mut query {
        tracker.update(current.effects());
    }
}
