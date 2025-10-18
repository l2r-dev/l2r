use bevy::prelude::*;
use game_core::stats::*;
use rand::Rng;
use spatial::{HeightDifference, RelativeDirection, TransformRelativeDirection};

pub fn calc_hit_miss(attacker: EntityRef, target: EntityRef, world: &World) -> bool {
    let accuracy = attacker
        .get::<AttackStats>()
        .map(|s| s.get(AttackStat::Accuracy))
        .unwrap_or_default();

    let evasion = target
        .get::<DefenceStats>()
        .map(|s| s.get(DefenceStat::Evasion))
        .unwrap_or_default();

    let condition_bonus = get_condition_bonus(attacker, target, world);

    calculate_hit_miss(accuracy, evasion, condition_bonus)
}

#[inline]
fn calculate_hit_chance(accuracy: f32, evasion: f32, condition_bonus: f32) -> f32 {
    // base_chance = 80% + 2% per accuracy advantage
    let mut chance = 80.0 + 2.0 * (accuracy - evasion);
    chance *= condition_bonus;
    chance = chance.clamp(20.0, 98.0);

    chance
}

#[inline]
fn calculate_hit_miss(accuracy: f32, evasion: f32, condition_bonus: f32) -> bool {
    let chance = calculate_hit_chance(accuracy, evasion, condition_bonus);
    let roll = rand::thread_rng().gen_range(0.0..100.0);
    chance <= roll
}

const HIGH_BONUS: f32 = 3.0;
const LOW_BONUS: f32 = -3.0;
const BACK_BONUS: f32 = 10.0;
const FRONT_BONUS: f32 = 0.0;
const SIDE_BONUS: f32 = 5.0;
const DARK_BONUS: f32 = -10.0;
const RAIN_BONUS: f32 = -3.0;

fn get_condition_bonus(attacker: EntityRef, target: EntityRef, _world: &World) -> f32 {
    get_condition_bonus_inner(attacker, target, _world).unwrap_or(1.0)
}

#[inline]
fn get_condition_bonus_inner(
    attacker: EntityRef,
    target: EntityRef,
    _world: &World,
) -> Option<f32> {
    let attacker_transform = attacker.get::<Transform>()?;
    let target_transform = target.get::<Transform>()?;

    // TODO: implement getting night/rain from world
    let night = false;
    let rain = false;

    Some(calculate_condition_bonus(attacker_transform, target_transform, night, rain).max(0.0))
}

fn calculate_condition_bonus(
    attacker: &Transform,
    target: &Transform,
    night: bool,
    rain: bool,
) -> f32 {
    let mut total_bonus = 100.0;

    if attacker
        .translation
        .significant_higher_than(&target.translation)
    {
        total_bonus += HIGH_BONUS;
    } else if attacker
        .translation
        .significant_lower_than(&target.translation)
    {
        total_bonus += LOW_BONUS;
    }

    if night {
        total_bonus += DARK_BONUS;
    }

    if rain {
        total_bonus += RAIN_BONUS;
    }

    let relative_dir = attacker.relative_direction(target);
    match relative_dir {
        RelativeDirection::Back => total_bonus += BACK_BONUS,
        RelativeDirection::Face => total_bonus += FRONT_BONUS,
        RelativeDirection::Side => total_bonus += SIDE_BONUS,
    }

    (total_bonus / 100.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;

    #[test]
    fn test_calculate_hit_miss_basic() {
        assert_eq!(calculate_hit_chance(10.0, 10.0, 1.0), 80.0);
        assert_eq!(calculate_hit_chance(20.0, 10.0, 1.0), 98.0);
        assert_eq!(calculate_hit_chance(10.0, 100.0, 1.0), 20.0);
        assert_eq!(calculate_hit_chance(33.0, 20.0, 1.0), 98.0);
    }

    #[test]
    fn test_calculate_condition_bonus_basic() {
        let mut attacker_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let mut target_transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));

        // Face each other to not calculate back/side bonuses
        attacker_transform.look_at(target_transform.translation, Vec3::Y);
        target_transform.look_at(attacker_transform.translation, Vec3::Y);

        // No special conditions
        let bonus = calculate_condition_bonus(&attacker_transform, &target_transform, false, false);
        assert_eq!(bonus, 1.0);

        // Night condition
        let night_bonus =
            calculate_condition_bonus(&attacker_transform, &target_transform, true, false);
        assert!(night_bonus < 1.0);

        // Rain condition
        let rain_bonus =
            calculate_condition_bonus(&attacker_transform, &target_transform, false, true);
        assert!(rain_bonus < 1.0);
    }

    #[test]
    fn test_calculate_condition_bonus_height_difference() {
        let high_attacker = Transform::from_translation(Vec3::new(0.0, 100.0, 0.0));
        let low_target = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

        let bonus = calculate_condition_bonus(&high_attacker, &low_target, false, false);
        assert!(bonus > 1.0);
    }
}
