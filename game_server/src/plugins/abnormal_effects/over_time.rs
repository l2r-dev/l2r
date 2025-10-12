use bevy::{ecs::system::SystemParam, log, prelude::*};
use game_core::{
    abnormal_effects::{AbnormalEffects, AbnormalEffectsTimers},
    stats::*,
};
use std::time::Duration;

#[derive(SystemParam)]
pub struct AbnormalEffectQueries<'w, 's> {
    effects: Query<
        'w,
        's,
        (
            Entity,
            Mut<'static, AbnormalEffects>,
            Mut<'static, AbnormalEffectsTimers>,
        ),
    >,
    vitals_stats: Query<'w, 's, Mut<'static, VitalsStats>>,
    attack_stats: Query<'w, 's, Mut<'static, AttackStats>>,
    defence_stats: Query<'w, 's, Mut<'static, DefenceStats>>,
    moveables: Query<'w, 's, Mut<'static, Movable>>,
    critical_stats: Query<'w, 's, Mut<'static, CriticalStats>>,
    primal_stats: Query<'w, 's, Mut<'static, PrimalStats>>,
    inventory_stats: Query<'w, 's, Mut<'static, InventoryStats>>,
    progress_stats: Query<'w, 's, Mut<'static, ProgressStats>>,
    progress_level_stats: Query<'w, 's, Mut<'static, ProgressLevelStats>>,
    progress_rates_stats: Query<'w, 's, Mut<'static, ProgressRatesStats>>,
    other_stats: Query<'w, 's, Mut<'static, OtherStats>>,
}

pub fn handle_abnormal_effect_effects_over_time(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut queries: AbnormalEffectQueries,
) -> Result<()> {
    use game_core::stats::StatKind::*;
    let time_spent = time.elapsed_secs() - *last_time;
    if time_spent >= 0.1 {
        *last_time = time.elapsed_secs();

        for (entity, mut abnormal_effects, mut timers) in queries.effects.iter_mut() {
            let mut remove_candidates = vec![];
            for (skill_id, eot) in timers.effects_over_time_mut() {
                eot.timer_mut().tick(Duration::from_secs_f32(time_spent));
                if let Vitals(vitals_stat) = eot.as_ref()
                    && queries.vitals_stats.get_mut(entity)?.get(vitals_stat) <= 0.0
                {
                    remove_candidates.push(skill_id);
                }
                if eot.timer().finished() {
                    match eot.as_ref() {
                        Vitals(vitals_stat) => {
                            queries
                                .vitals_stats
                                .get_mut(entity)?
                                .apply_operation(vitals_stat, eot.as_ref());
                        }
                        Attack(attack_stat) => {
                            queries
                                .attack_stats
                                .get_mut(entity)?
                                .apply_operation(attack_stat, eot.as_ref());
                        }
                        Defence(defence_stat) => {
                            queries
                                .defence_stats
                                .get_mut(entity)?
                                .apply_operation(defence_stat, eot.as_ref());
                        }
                        Movement(movement_stat) => {
                            queries
                                .moveables
                                .get_mut(entity)?
                                .movement_stats_mut()
                                .apply_operation(movement_stat, &eot.operation().convert()?);
                        }
                        Critical(critical_stat) => {
                            queries
                                .critical_stats
                                .get_mut(entity)?
                                .apply_operation(critical_stat, &eot.operation().convert()?);
                        }
                        Primal(primal_stat) => {
                            queries
                                .primal_stats
                                .get_mut(entity)?
                                .apply_operation(primal_stat, &eot.operation().convert()?);
                        }
                        ElementPower(element) => {
                            log::warn!("ElementPower not yet implemented: {:?}", element);
                        }
                        Inventory(inventory_stat) => {
                            queries
                                .inventory_stats
                                .get_mut(entity)?
                                .apply_operation(inventory_stat, &eot.operation().convert()?);
                        }
                        MpConsumption(mp_consumption_stat) => {
                            log::warn!(
                                "MpConsumption not yet implemented: {:?}",
                                mp_consumption_stat
                            );
                        }
                        Progress(progress_stat) => {
                            queries
                                .progress_stats
                                .get_mut(entity)?
                                .apply_operation(progress_stat, &eot.operation().convert()?);
                        }
                        ProgressLevel(progress_level_stat) => {
                            queries
                                .progress_level_stats
                                .get_mut(entity)?
                                .apply_operation(progress_level_stat, &eot.operation().convert()?);
                        }
                        ProgressRates(progress_rates_stat) => {
                            queries
                                .progress_rates_stats
                                .get_mut(entity)?
                                .apply_operation(progress_rates_stat, eot.as_ref());
                        }
                        Other(other_stat) => {
                            queries
                                .other_stats
                                .get_mut(entity)?
                                .apply_operation(other_stat, eot.as_ref());
                        }
                    };
                }
            }

            remove_candidates.into_iter().for_each(|skill_id| {
                abnormal_effects.remove(skill_id);
                timers.remove(skill_id);
            });
        }
    }
    Ok(())
}
