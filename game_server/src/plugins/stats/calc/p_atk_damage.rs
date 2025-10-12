use crate::plugins::stats::get_attack_trait_bonus;
use bevy::prelude::*;
use config::Config;
use game_core::{
    character::Character,
    items::{DollSlot, ItemsDataTable, ItemsInfo, Kind, PaperDoll},
    npc::Kind as NpcKind,
    stats::*,
};
use rand::Rng;
use spatial::TransformRelativeDirection;

pub fn calc_p_atk_damage(
    attacker: EntityRef,
    target: EntityRef,
    world: &World,
    critical: bool,
    soulshot_used: bool,
    shield_result: ShieldResult,
) -> f32 {
    let Some(attack_stats) = attacker.get::<AttackStats>() else {
        return 0.0;
    };
    let Some(defence_stats) = target.get::<DefenceStats>() else {
        return 0.0;
    };
    let Some(attacker_critical_stats) = attacker.get::<CriticalStats>() else {
        return 0.0;
    };

    let mut p_atk = attack_stats.get(&AttackStat::PAtk);
    let mut p_def = defence_stats.get(&DefenceStat::PDef);

    // Check if it's PvP or PvE
    let is_pvp = attacker.get::<Character>().is_some() && target.get::<Character>().is_some();
    let is_pve = attacker.get::<Character>().is_some() && target.get::<NpcKind>().is_some();

    if is_pvp {
        p_def *= defence_stats.get(&DefenceStat::PvpPDefBonus);
    }

    match shield_result {
        ShieldResult::Succeed => {
            p_def += defence_stats.get(&DefenceStat::ShieldDefence);
        }
        ShieldResult::PerfectBlock => {
            return 1.0; // Perfect block always deals 1 damage
        }
        _ => {}
    }

    p_atk *= if soulshot_used { 2.0 } else { 1.0 };

    let relative_dir = attacker
        .get::<Transform>()
        .and_then(|attacker_transform| {
            target
                .get::<Transform>()
                .map(|target_transform| attacker_transform.relative_direction(target_transform))
        })
        .unwrap_or_default();

    let base_damage = (76.0 * p_atk * relative_dir.attack_bonus()) / p_def;

    let mut damage = if critical {
        let crit_damage_base = attacker_critical_stats.get(&CriticalStat::CriticalDamage);
        let defence_crit_damage = defence_stats.get(&DefenceStat::DefenceCriticalDamage);

        let mut critical_damage = 2.0
            * crit_damage_base
            * attacker_critical_stats.positional_damage(relative_dir)
            * defence_crit_damage
            * base_damage;

        let crit_damage_add = attacker_critical_stats.get(&CriticalStat::CriticalDamageAdditional);
        critical_damage += (crit_damage_add * 77.0) / p_def;

        critical_damage += defence_stats.get(&DefenceStat::DefenceCriticalDamageAdditional);

        critical_damage
    } else {
        base_damage
    };

    damage *= get_attack_trait_bonus(attacker, target);

    damage *= get_random_damage_multiplier(attacker, world);

    if damage > 0.0 && damage < 1.0 {
        damage = 1.0;
    } else if damage < 0.0 {
        damage = 0.0;
    }

    if is_pvp {
        damage *= attack_stats.get(&AttackStat::PvpPAtkBonus);
    }

    // TODO: Apply attribute bonus
    // damage *= calc_attribute_bonus(attacker, target, world);

    // Apply PvE bonuses
    if is_pve {
        damage = apply_pve_bonuses(attacker, target, world, damage, critical);
    }

    damage
}

fn get_random_damage_multiplier(attacker: EntityRef, world: &World) -> f32 {
    // Get random damage multiplier from weapon
    if let Some(paper_doll) = attacker.get::<PaperDoll>()
        && let Some(weapon) = paper_doll.get(DollSlot::RightHand)
        && let (Some(table), Some(assets)) = (
            world.get_resource::<ItemsDataTable>(),
            world.get_resource::<Assets<ItemsInfo>>(),
        )
        && let Ok(item_info) = table.get_item_info(weapon.item().id(), assets)
        && let Kind::Weapon(weapon_data) = item_info.kind()
    {
        let random = weapon_data.random_damage as i32;
        return 1.0 + (rand::thread_rng().gen_range(-random..=random) as f32 / 100.0);
    }

    // Fallback if can't get from weapon: use level-based calculation
    let level: f32 = attacker
        .get::<ProgressLevelStats>()
        .map(|stats| stats.level().into())
        .unwrap_or(1.0);

    let random = 5 + level.sqrt() as i32;
    1.0 + (rand::thread_rng().gen_range(-random..=random) as f32 / 100.0)
}

fn apply_pve_bonuses(
    attacker: EntityRef,
    target: EntityRef,
    world: &World,
    mut damage: f32,
    critical: bool,
) -> f32 {
    let Some(attack_stats) = attacker.get::<AttackStats>() else {
        return damage;
    };

    if let Some(paper_doll) = attacker.get::<PaperDoll>() {
        if let Some(weapon) = paper_doll.get(DollSlot::RightHand) {
            let items_data_table = world.get_resource::<ItemsDataTable>();
            let items_data_assets = world.get_resource::<Assets<ItemsInfo>>();

            if let (Some(table), Some(assets)) = (items_data_table, items_data_assets)
                && let Ok(item_info) = table.get_item_info(weapon.item().id(), assets)
            {
                if item_info.kind().bow_or_crossbow() {
                    damage *= attack_stats.get(&AttackStat::PveBowPAtkBonus);
                } else {
                    damage *= attack_stats.get(&AttackStat::PvePAtkBonus);
                }
            }
        } else {
            // No weapon equipped
            damage *= attack_stats.get(&AttackStat::PvePAtkBonus);
        }
    } else {
        // No paper doll
        damage *= attack_stats.get(&AttackStat::PvePAtkBonus);
    }

    damage = apply_level_diff_penalty(attacker, target, world, damage, critical);

    damage
}

fn apply_level_diff_penalty(
    attacker: EntityRef,
    target: EntityRef,
    world: &World,
    mut damage: f32,
    critical: bool,
) -> f32 {
    let Some(config) = world.get_resource::<Config>() else {
        return damage;
    };

    let attacker_level = attacker
        .get::<ProgressLevelStats>()
        .map(|stats| stats.level())
        .unwrap_or_default();

    let target_level = target
        .get::<ProgressLevelStats>()
        .map(|stats| stats.level())
        .unwrap_or_default();

    if let Some(npc_kind) = target.get::<NpcKind>() {
        let is_raid = matches!(npc_kind, NpcKind::RaidBoss | NpcKind::RaidMinion);
        let is_raid_minion = matches!(npc_kind, NpcKind::RaidMinion);

        let min_npc_level_dmg_penalty = config.gameplay().min_npc_level_dmg_penalty;
        let level_diff = target_level.diff(&attacker_level);

        if !is_raid
            && !is_raid_minion
            && *target_level >= min_npc_level_dmg_penalty
            && level_diff >= 2
        {
            let level_diff_index = (level_diff as usize).saturating_sub(2);

            let penalty = if critical {
                let penalties = config.gameplay().npc_crit_dmg_penalty.as_slice();
                if level_diff_index < penalties.len() {
                    penalties[level_diff_index]
                } else {
                    penalties.last().copied().unwrap_or_default()
                }
            } else {
                let penalties = config.gameplay().npc_dmg_penalty.as_slice();
                if level_diff_index < penalties.len() {
                    penalties[level_diff_index]
                } else {
                    penalties.last().copied().unwrap_or_default()
                }
            };

            damage *= penalty;
        }
    }

    damage
}
