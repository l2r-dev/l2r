use crate::plugins::stats::{EffectQuery, calc::effect_kind::get_attack_trait_bonus};
use bevy::prelude::*;
use bevy_ecs::system::SystemParam;
use config::Config;
use game_core::{
    character::Character,
    items::{DollSlot, ItemsDataQuery, Kind, PaperDoll},
    npc::Kind as NpcKind,
    stats::*,
};
use rand::Rng;
use spatial::TransformRelativeDirection;

#[derive(SystemParam)]
pub struct PAtkCalcDamageQuery<'w, 's> {
    pub transforms: Query<'w, 's, Ref<'static, Transform>>,
    pub attack_stats: Query<'w, 's, Ref<'static, AttackStats>>,
    pub defence_stats: Query<'w, 's, Ref<'static, DefenceStats>>,
    pub crit_stats: Query<'w, 's, Ref<'static, CriticalStats>>,
    pub characters: Query<'w, 's, Ref<'static, Character>>,
    pub npc_kinds: Query<'w, 's, Ref<'static, NpcKind>>,
    pub paper_dolls: Query<'w, 's, Ref<'static, PaperDoll>>,
    pub level_stats: Query<'w, 's, Ref<'static, ProgressLevelStats>>,
    pub atk_def_effects: EffectQuery<'w, 's>,
    pub config: Res<'w, Config>,
    pub items_data_query: ItemsDataQuery<'w>,
}

pub fn calc_p_atk_damage(
    attacker: Entity,
    target: Entity,
    critical: bool,
    soulshot_used: bool,
    shield_result: ShieldResult,
    query: &PAtkCalcDamageQuery,
) -> f32 {
    let Some(attack_stats) = query.attack_stats.get(attacker).ok() else {
        return 0.0;
    };
    let Some(defence_stats) = query.defence_stats.get(target).ok() else {
        return 0.0;
    };
    let Some(attacker_critical_stats) = query.crit_stats.get(attacker).ok() else {
        return 0.0;
    };

    let mut p_atk = attack_stats.get(AttackStat::PAtk);
    let mut p_def = defence_stats.get(DefenceStat::PDef);

    // Check if it's PvP or PvE
    let is_pvp = query.characters.get(attacker).is_ok() && query.characters.get(target).is_ok();
    let is_pve = query.characters.get(attacker).is_ok() && query.npc_kinds.get(target).is_ok();

    if is_pvp {
        p_def *= defence_stats.get(DefenceStat::PvpPDefBonus);
    }

    match shield_result {
        ShieldResult::Succeed => {
            p_def += defence_stats.get(DefenceStat::ShieldDefence);
        }
        ShieldResult::PerfectBlock => {
            return 1.0; // Perfect block always deals 1 damage
        }
        _ => {}
    }

    p_atk *= if soulshot_used { 2.0 } else { 1.0 };

    let relative_dir = query
        .transforms
        .get(attacker)
        .ok()
        .and_then(|attacker_transform| {
            query.transforms.get(target).ok().map(|target_transform| {
                attacker_transform.relative_direction(target_transform.as_ref())
            })
        })
        .unwrap_or_default();

    let base_damage = (76.0 * p_atk * relative_dir.attack_bonus()) / p_def;

    let mut damage = if critical {
        let crit_damage_base = attacker_critical_stats.get(CriticalStat::CriticalDamage);
        let defence_crit_damage = defence_stats.get(DefenceStat::DefenceCriticalDamage);

        let mut critical_damage = 2.0
            * crit_damage_base
            * attacker_critical_stats.positional_damage(relative_dir)
            * defence_crit_damage
            * base_damage;

        let crit_damage_add = attacker_critical_stats.get(CriticalStat::CriticalDamageAdditional);
        critical_damage += (crit_damage_add * 77.0) / p_def;

        critical_damage += defence_stats.get(DefenceStat::DefenceCriticalDamageAdditional);

        critical_damage
    } else {
        base_damage
    };

    damage *= get_attack_trait_bonus(attacker, target, &query.atk_def_effects);

    damage *= get_random_damage_multiplier(attacker, query);

    if damage > 0.0 && damage < 1.0 {
        damage = 1.0;
    } else if damage < 0.0 {
        damage = 0.0;
    }

    if is_pvp {
        damage *= attack_stats.get(AttackStat::PvpPAtkBonus);
    }

    // TODO: Apply attribute bonus
    // damage *= calc_attribute_bonus(attacker, target, world);

    // Apply PvE bonuses
    if is_pve {
        damage = apply_pve_bonuses(attacker, target, query, damage, critical);
    }

    damage
}

fn get_random_damage_multiplier(attacker: Entity, query: &PAtkCalcDamageQuery) -> f32 {
    // Get random damage multiplier from weapon
    if let Ok(paper_doll) = query.paper_dolls.get(attacker)
        && let Some(weapon) = paper_doll.get(DollSlot::RightHand)
        && let Ok(item_info) = query.items_data_query.get_item_info(weapon.item().id())
        && let Kind::Weapon(weapon_data) = item_info.kind()
    {
        let random = weapon_data.random_damage as i32;
        return 1.0 + (rand::thread_rng().gen_range(-random..=random) as f32 / 100.0);
    }

    // Fallback if can't get from weapon: use level-based calculation
    let level: f32 = query
        .level_stats
        .get(attacker)
        .map(|stats| stats.level().into())
        .unwrap_or(1.0);

    let random = 5 + level.sqrt() as i32;
    1.0 + (rand::thread_rng().gen_range(-random..=random) as f32 / 100.0)
}

fn apply_pve_bonuses(
    attacker: Entity,
    target: Entity,
    query: &PAtkCalcDamageQuery,
    mut damage: f32,
    critical: bool,
) -> f32 {
    let Some(attack_stats) = query.attack_stats.get(attacker).ok() else {
        return damage;
    };

    if let Ok(paper_doll) = query.paper_dolls.get(attacker) {
        if let Some(weapon) = paper_doll.get(DollSlot::RightHand)
            && let Ok(item_info) = query.items_data_query.get_item_info(weapon.item().id())
        {
            if item_info.kind().bow_or_crossbow() {
                damage *= attack_stats.get(AttackStat::PveBowPAtkBonus);
            } else {
                damage *= attack_stats.get(AttackStat::PvePAtkBonus);
            }
        } else {
            // No weapon equipped
            damage *= attack_stats.get(AttackStat::PvePAtkBonus);
        }
    } else {
        // No paper doll
        damage *= attack_stats.get(AttackStat::PvePAtkBonus);
    }

    damage = apply_level_diff_penalty(attacker, target, query, damage, critical);

    damage
}

fn apply_level_diff_penalty(
    attacker: Entity,
    target: Entity,
    query: &PAtkCalcDamageQuery,
    mut damage: f32,
    critical: bool,
) -> f32 {
    let attacker_level = query
        .level_stats
        .get(attacker)
        .map(|stats| stats.level())
        .unwrap_or_default();

    let target_level = query
        .level_stats
        .get(target)
        .map(|stats| stats.level())
        .unwrap_or_default();

    if let Ok(npc_kind) = query.npc_kinds.get(target) {
        let is_raid = matches!(*npc_kind, NpcKind::RaidBoss | NpcKind::RaidMinion);
        let is_raid_minion = matches!(*npc_kind, NpcKind::RaidMinion);

        let min_npc_level_dmg_penalty = query.config.gameplay().min_npc_level_dmg_penalty;
        let level_diff = target_level.diff(&attacker_level);

        if !is_raid
            && !is_raid_minion
            && *target_level >= min_npc_level_dmg_penalty
            && level_diff >= 2
        {
            let level_diff_index = (level_diff as usize).saturating_sub(2);

            let penalty = if critical {
                let penalties = query.config.gameplay().npc_crit_dmg_penalty.as_slice();
                if level_diff_index < penalties.len() {
                    penalties[level_diff_index]
                } else {
                    penalties.last().copied().unwrap_or_default()
                }
            } else {
                let penalties = query.config.gameplay().npc_dmg_penalty.as_slice();
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
