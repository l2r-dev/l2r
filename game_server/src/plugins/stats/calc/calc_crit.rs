use bevy::prelude::*;
use bevy_ecs::system::SystemParam;
use config::Config;
use game_core::stats::*;
use rand::Rng;
use spatial::{RelativeDirection, TransformRelativeDirection};

#[derive(SystemParam)]
pub struct CalcCritQuery<'w, 's> {
    pub transforms: Query<'w, 's, Ref<'static, Transform>>,
    pub crit_stats: Query<'w, 's, Ref<'static, CriticalStats>>,
    pub defence_stats: Query<'w, 's, Ref<'static, DefenceStats>>,
    pub config: Res<'w, Config>,
}

pub fn calc_crit(attacker: Entity, target: Entity, query: &CalcCritQuery) -> bool {
    let rate = calculate_crit_rate(attacker, target, query);
    let max_crit_rate = query.config.gameplay().max_crit_rate;
    let rate = rate.clamp(1.0, max_crit_rate as f32);
    let defence_rate = calculate_defence_crit_rate(target, query);
    calculate_crit_result(rate, defence_rate)
}

fn calculate_crit_rate(attacker: Entity, target: Entity, query: &CalcCritQuery) -> f32 {
    let base_crit_rate = query
        .crit_stats
        .get(attacker)
        .map(|s| s.get(CriticalStat::CriticalRate))
        .unwrap_or_default();

    let attacker_transform = query.transforms.get(attacker).ok();
    let target_transform = query.transforms.get(target).ok();

    if let (Some(attacker_transform), Some(target_transform)) =
        (attacker_transform, target_transform)
    {
        let relative_dir = attacker_transform.relative_direction(target_transform.as_ref());

        let positional_crit_rate = query
            .crit_stats
            .get(attacker)
            .map(|s| match relative_dir {
                RelativeDirection::Face => s.get(CriticalStat::CriticalRateFront),
                RelativeDirection::Back => s.get(CriticalStat::CriticalRateBack),
                RelativeDirection::Side => s.get(CriticalStat::CriticalRateSide),
            })
            .unwrap_or(1.0);

        base_crit_rate * positional_crit_rate
    } else {
        base_crit_rate
    }
}

fn calculate_defence_crit_rate(target: Entity, query: &CalcCritQuery) -> f32 {
    let defence_stats = query.defence_stats.get(target).unwrap();
    let defence_crit_rate = defence_stats.get(DefenceStat::DefenceCriticalRate);
    let defence_crit_rate_add = defence_stats.get(DefenceStat::DefenceCriticalRateAdditional);
    defence_crit_rate + defence_crit_rate_add
}

#[inline]
fn calculate_crit_result(crit_rate: f32, defence_rate: f32) -> bool {
    let final_rate = (crit_rate + defence_rate).max(0.0);
    let roll = rand::thread_rng().gen_range(0..1000);
    final_rate > roll as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_crit_result_basic() {
        assert!(calculate_crit_result(1001.0, 0.0)); // 100% crit chance
        assert!(!calculate_crit_result(0.0, -1000.0)); // 0% crit chance
    }
}
