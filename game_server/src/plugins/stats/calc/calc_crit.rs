use bevy::prelude::*;
use config::Config;
use game_core::stats::*;
use rand::Rng;
use spatial::{RelativeDirection, TransformRelativeDirection};

pub fn calc_crit(attacker: EntityRef, target: EntityRef, world: &World) -> bool {
    let config = world.resource::<Config>();

    let rate = calculate_crit_rate(attacker, target);

    let max_crit_rate = config.gameplay().max_crit_rate;
    let rate = rate.clamp(1.0, max_crit_rate as f32);

    let defence_rate = calculate_defence_crit_rate(target);

    calculate_crit_result(rate, defence_rate)
}

fn calculate_crit_rate(attacker_entity: EntityRef, target_entity: EntityRef) -> f32 {
    let base_crit_rate = attacker_entity
        .get::<CriticalStats>()
        .map(|s| s.get(&CriticalStat::CriticalRate))
        .unwrap_or_default();

    let attacker_transform = attacker_entity.get::<Transform>();
    let target_transform = target_entity.get::<Transform>();

    if let (Some(attacker_transform), Some(target_transform)) =
        (attacker_transform, target_transform)
    {
        let relative_dir = attacker_transform.relative_direction(target_transform);

        let positional_crit_rate = attacker_entity
            .get::<CriticalStats>()
            .map(|s| match relative_dir {
                RelativeDirection::Face => s.get(&CriticalStat::CriticalRateFront),
                RelativeDirection::Back => s.get(&CriticalStat::CriticalRateBack),
                RelativeDirection::Side => s.get(&CriticalStat::CriticalRateSide),
            })
            .unwrap_or(1.0);

        base_crit_rate * positional_crit_rate
    } else {
        base_crit_rate
    }
}

fn calculate_defence_crit_rate(target_entity: EntityRef) -> f32 {
    let defence_stats = target_entity.get::<DefenceStats>();

    let defence_crit_rate = defence_stats
        .map(|s| s.get(&DefenceStat::DefenceCriticalRate))
        .unwrap_or_default();

    let defence_crit_rate_add = defence_stats
        .map(|s| s.get(&DefenceStat::DefenceCriticalRateAdditional))
        .unwrap_or_default();

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
