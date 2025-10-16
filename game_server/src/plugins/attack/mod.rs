mod death;
mod packet;

use crate::plugins::stats::{
    calc_crit, calc_hit_miss, calc_p_atk_damage, calculate_shield_result,
    send_shield_result_system_message,
};
use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::ParallelCommands,
    },
    prelude::*,
};
use game_core::{
    animation::{Animation, AnimationTimer},
    attack::{
        AttackAllowed, AttackComponentsPlugin, AttackHit, AttackTimer, Attacking, AttackingList,
        Dead, HitInfo, InCombat,
    },
    items,
    items::{
        BluntType, DaggerType, DollSlot, Grade, Item, ItemsDataQuery, Kind, PaperDoll, Soulshot,
        SwordType, WeaponKind,
    },
    movement::{LookAt, MoveToEntity},
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{Attack, GameServerPacket, SystemMessage},
    },
    npc,
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId},
    path_finding::{InActionPathfindingTimer, VisibilityCheckRequest},
    stats::*,
};
use map::{WorldMap, WorldMapQuery};
use spatial::{FlatDistance, TransformRelativeDirection};
use state::GameMechanicsSystems;
use system_messages;

pub struct AttackPlugin;
impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AttackComponentsPlugin);

        app.add_plugins(packet::AttackPacketPlugin)
            .add_plugins(death::DeathPlugin);

        app.add_systems(
            FixedUpdate,
            attack_entity.in_set(GameMechanicsSystems::Attacking),
        )
        .add_systems(
            FixedUpdate,
            process_attack_hits.in_set(GameMechanicsSystems::Attacking),
        )
        .add_systems(
            FixedUpdate,
            setup_attack_timers.in_set(GameMechanicsSystems::Attacking),
        )
        .add_systems(
            FixedUpdate,
            calculate_attack_allowed.in_set(GameMechanicsSystems::Attacking),
        )
        .add_systems(
            Update,
            in_combat_handler.in_set(GameMechanicsSystems::Attacking),
        );
    }
}

#[derive(QueryData)]
struct AttackingQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    attack_stats: Ref<'a, AttackStats>,
    transform: Ref<'a, Transform>,
    target: Ref<'a, Attacking>,
    paper_doll: Option<Ref<'a, PaperDoll>>,
    move_to: Option<Ref<'a, MoveToEntity>>,
}

#[derive(QueryFilter)]
struct AttackingFilter {
    attack_allowed: With<AttackAllowed>,
    not_dead: Without<Dead>,
    // without_pending_skill: Without<PendingSkill>,
    not_animating: Without<Animation>,
    not_pathfinding: Without<InActionPathfindingTimer>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct EnemyQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

fn attack_entity(
    attacking_objects: Query<AttackingQuery, AttackingFilter>,
    target_objects: Query<EnemyQuery, Without<Dead>>,
    world: &World,
    shot_used: Query<Ref<Soulshot>>,
    par_commands: ParallelCommands,
    items: Query<Entity, With<Item>>,
    items_data_query: ItemsDataQuery,
    object_id_manager: Res<ObjectIdManager>,
    world_map_query: WorldMapQuery,
) -> Result<()> {
    attacking_objects.par_iter().for_each(|attacker| {
        match target_objects.get(**attacker.target) {
            Ok(aiming_target) => {
                let distance = attacker
                    .transform
                    .translation
                    .flat_distance(&aiming_target.transform.translation);

                let range = attacker.attack_stats.get(&AttackStat::PAtkRange);
                if distance <= range {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(LookAt(aiming_target.entity), attacker.entity);
                        // Remove the AttackAllowed component to ensure the entity waits for
                        // the next allowed attack time
                        commands
                            .entity(attacker.entity)
                            .remove::<AttackAllowed>()
                            .try_insert(InCombat::default());
                        commands
                            .entity(attacker.entity)
                            .remove::<MoveToEntity>()
                            .try_insert(InCombat::default());
                        commands
                            .entity(aiming_target.entity)
                            .try_insert(InCombat::default());
                    });

                    let Ok(attacker_ref) = world.get_entity(attacker.entity) else {
                        return;
                    };

                    let Ok(aiming_target_ref) = world.get_entity(aiming_target.entity) else {
                        return;
                    };

                    let attack_interval = attacker
                        .attack_stats
                        .typed::<PAtkSpd>(&AttackStat::PAtkSpd)
                        .attack_interval();

                    let weapon = attacker
                        .paper_doll
                        .as_ref()
                        .and_then(|paper_doll| paper_doll.get(DollSlot::RightHand));

                    let mut second_attack_interval_multiplier = None;

                    let interval_multiplier = if let Some(weapon) = weapon
                        && let Some(items_data_assets) =
                            world.get_resource::<Assets<items::ItemsInfo>>()
                        && let Some(items_data_table) =
                            world.get_resource::<items::ItemsDataTable>()
                        && let Ok(item_info) =
                            items_data_table.get_item_info(weapon.item().id(), items_data_assets)
                        && let Kind::Weapon(weapon) = item_info.kind()
                    {
                        match weapon.kind {
                            WeaponKind::Sword(sword) => {
                                if matches!(sword, SwordType::Dual) {
                                    second_attack_interval_multiplier = Some(0.2);

                                    0.4
                                } else {
                                    0.5
                                }
                            }

                            WeaponKind::Blunt(blunt) => {
                                if matches!(blunt, BluntType::Dual) {
                                    second_attack_interval_multiplier = Some(0.2);

                                    0.4
                                } else {
                                    0.5
                                }
                            }

                            WeaponKind::Dagger(dagger) => {
                                if matches!(dagger, DaggerType::Dual) {
                                    second_attack_interval_multiplier = Some(0.2);

                                    0.4
                                } else {
                                    0.5
                                }
                            }

                            WeaponKind::Fist(_) => {
                                second_attack_interval_multiplier = Some(0.2);

                                0.4
                            }

                            WeaponKind::Pole => 0.6,

                            WeaponKind::Bow
                            | WeaponKind::Crossbow
                            | WeaponKind::Etc
                            | WeaponKind::FortFlag
                            | WeaponKind::FishingRod => 0.5,
                        }
                    } else {
                        0.5
                    };

                    let weapon_entity = weapon.and_then(|weapon| {
                        items
                            .by_object_id(weapon.object_id(), object_id_manager.as_ref())
                            .ok()
                    });

                    let soulshot_used = weapon_entity.and_then(|v| shot_used.get(v).ok()).is_some();

                    let soulshot_grade = if let Some(w) = weapon
                        && let Ok(it) = items_data_query.get_item_info(w.item().id())
                    {
                        it.grade()
                    } else {
                        Grade::None
                    };

                    let mut attack_info = Attack::new(
                        *attacker.object_id,
                        attacker.transform.translation,
                        aiming_target.transform.translation,
                    );

                    let mut max_targets_count = attacker_ref
                        .get::<AttackStats>()
                        .map(|s| s.get(&AttackStat::PAtkMaxTargetsCount))
                        .unwrap_or_default()
                        .round() as u32;

                    let attack_hit = if max_targets_count > 1 {
                        let mut hits = vec![];

                        let hit_info =
                            calc_hit_info(soulshot_used, attacker_ref, aiming_target_ref, world);

                        attack_info.add_hit(
                            hit_info.dmg as u32,
                            *aiming_target.object_id,
                            hit_info.miss,
                            hit_info.crit,
                            0,
                            soulshot_used,
                            soulshot_grade,
                        );

                        hits.push((aiming_target.entity, hit_info));

                        max_targets_count -= 1;

                        let angle = attacker_ref
                            .get::<AttackStats>()
                            .map(|s| s.get(&AttackStat::PAtkWidth))
                            .unwrap_or_default()
                            .round();

                        for next_target in &target_objects {
                            if next_target.entity == aiming_target.entity {
                                continue;
                            }

                            if next_target.entity == attacker.entity {
                                continue;
                            }

                            let Ok(next_target_ref) = world.get_entity(aiming_target.entity) else {
                                continue;
                            };

                            let distance = attacker
                                .transform
                                .translation
                                .flat_distance(&next_target.transform.translation);

                            if distance > range {
                                continue;
                            }

                            if !next_target
                                .transform
                                .is_within_angle_relative_to(&attacker.transform, angle)
                            {
                                continue;
                            }

                            //TODO: нужна проверка на враждебность

                            let hit_info =
                                calc_hit_info(soulshot_used, attacker_ref, next_target_ref, world);
                            attack_info.add_hit(
                                hit_info.dmg as u32,
                                *aiming_target.object_id,
                                hit_info.miss,
                                hit_info.crit,
                                0,
                                soulshot_used,
                                soulshot_grade,
                            );
                            hits.push((next_target.entity, hit_info));

                            if max_targets_count == 1 {
                                break;
                            } else {
                                max_targets_count -= 1;
                            }
                        }

                        AttackHit::new_multi(
                            attack_interval.mul_f32(interval_multiplier),
                            weapon_entity,
                            hits,
                        )
                    } else {
                        let hit_info =
                            calc_hit_info(soulshot_used, attacker_ref, aiming_target_ref, world);

                        attack_info.add_hit(
                            hit_info.dmg as u32,
                            *aiming_target.object_id,
                            hit_info.miss,
                            hit_info.crit,
                            0,
                            soulshot_used,
                            soulshot_grade,
                        );

                        if let Some(second_attack_interval_multiplier) =
                            second_attack_interval_multiplier
                        {
                            let second_hit_info = calc_hit_info(
                                soulshot_used,
                                attacker_ref,
                                aiming_target_ref,
                                world,
                            );

                            attack_info.add_hit(
                                second_hit_info.dmg as u32,
                                *aiming_target.object_id,
                                second_hit_info.miss,
                                second_hit_info.crit,
                                0,
                                soulshot_used,
                                soulshot_grade,
                            );

                            AttackHit::new_dual(
                                aiming_target.entity,
                                weapon_entity,
                                attack_interval.mul_f32(interval_multiplier),
                                hit_info,
                                attack_interval.mul_f32(second_attack_interval_multiplier),
                                second_hit_info,
                            )
                        } else {
                            AttackHit::new_common(
                                aiming_target.entity,
                                attack_interval.mul_f32(interval_multiplier),
                                hit_info,
                                weapon_entity,
                            )
                        }
                    };

                    par_commands.command_scope(|mut commands| {
                        commands.entity(attacker.entity).try_insert(attack_hit);
                    });

                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(
                            ServerPacketBroadcast::new(attack_info.into()),
                            attacker.entity,
                        );

                        commands
                            .entity(attacker.entity)
                            .try_insert(AnimationTimer::new(attack_interval));
                    });
                } else {
                    // Target is out of range, need to chase
                    // Check if already moving to the correct target
                    if let Some(move_to) = attacker.move_to
                        && move_to.target == aiming_target.entity
                    {
                        return;
                    }

                    let attacker_pos = attacker.transform.translation;
                    let target_pos = aiming_target.transform.translation;

                    let geodata = world_map_query
                        .region_geodata_from_pos(attacker_pos)
                        .unwrap();

                    // Use the same logic as follow plugin - check line of sight
                    let can_move_to = geodata.can_move_to(
                        &WorldMap::vec3_to_geo(attacker_pos),
                        &WorldMap::vec3_to_geo(target_pos),
                    );

                    if can_move_to {
                        // Direct line of sight, use simple movement
                        par_commands.command_scope(|mut commands| {
                            commands.entity(attacker.entity).try_insert(MoveToEntity {
                                target: aiming_target.entity,
                                range,
                            });
                        });
                    } else {
                        // No line of sight, use pathfinding
                        par_commands.command_scope(|mut commands| {
                            commands
                                .entity(attacker.entity)
                                .try_insert(InActionPathfindingTimer::default());

                            commands.trigger_targets(
                                VisibilityCheckRequest {
                                    entity: attacker.entity,
                                    start: attacker_pos,
                                    target: target_pos,
                                },
                                attacker.entity,
                            );
                        });
                    }
                }
            }
            _ => {
                par_commands.command_scope(|mut commands| {
                    commands.entity(attacker.entity).remove::<Attacking>();
                });
            }
        }
    });
    Ok(())
}
#[derive(QueryData)]
#[query_data(mutable)]
struct ProcessAttackHitsQuery<'a> {
    entity: Entity,
    name: Ref<'a, Name>,
    object_id: Ref<'a, ObjectId>,
    hit: Mut<'a, AttackHit>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ProcessAttackHitsTargetQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    vital_stats: Mut<'a, VitalsStats>,
}

fn process_attack_hits(
    time: Res<Time>,
    mut commands: Commands,

    mut pending_attackers: Query<ProcessAttackHitsQuery>,
    mut targets: Query<ProcessAttackHitsTargetQuery, Without<Dead>>,
    not_attacking_npc: Query<Entity, (With<npc::Kind>, Without<Attacking>)>,
    npc: Query<Entity, With<npc::Kind>>,

    mut attackers_lists: Query<Mut<AttackingList>>,
) {
    for attacker in pending_attackers.iter_mut() {
        let mut pending_hit = attacker.hit;

        pending_hit.timer_mut().tick(time.delta());

        if pending_hit.timer().finished() {
            let attacker_entity = attacker.entity;

            match &mut *pending_hit {
                AttackHit::AttackCommonHit(pending_hit) => {
                    if let Ok(mut target) = targets.get_mut(pending_hit.target()) {
                        let info = pending_hit.hit();

                        process_hit(
                            commands.reborrow(),
                            npc,
                            &mut attackers_lists,
                            info,
                            attacker.entity,
                            attacker.name.to_string(),
                            target.entity,
                            &mut target.vital_stats,
                            not_attacking_npc,
                        );
                    }

                    remove_soulshot(commands.reborrow(), pending_hit.weapon_entity());

                    commands.entity(attacker_entity).remove::<AttackHit>();
                }

                AttackHit::AttackMultiHit(pending_hits) => {
                    for (target, info) in pending_hits.hits() {
                        if let Ok(mut target) = targets.get_mut(*target) {
                            process_hit(
                                commands.reborrow(),
                                npc,
                                &mut attackers_lists,
                                *info,
                                attacker.entity,
                                attacker.name.to_string(),
                                target.entity,
                                &mut target.vital_stats,
                                not_attacking_npc,
                            );
                        }
                    }

                    remove_soulshot(commands.reborrow(), pending_hits.weapon_entity());

                    commands.entity(attacker_entity).remove::<AttackHit>();
                }

                AttackHit::AttackDualHit(pending_dual_hit) => {
                    if let Ok(mut target) = targets.get_mut(pending_dual_hit.target()) {
                        let info = pending_dual_hit.hit();

                        process_hit(
                            commands.reborrow(),
                            npc,
                            &mut attackers_lists,
                            info,
                            attacker.entity,
                            attacker.name.to_string(),
                            target.entity,
                            &mut target.vital_stats,
                            not_attacking_npc,
                        );
                    }

                    if pending_dual_hit.set_to_secondary() {
                        remove_soulshot(commands.reborrow(), pending_dual_hit.weapon_entity());
                    } else {
                        commands.entity(attacker_entity).remove::<AttackHit>();
                    }
                }
            }
        }
    }
}

fn process_hit(
    mut commands: Commands,
    npc: Query<Entity, With<npc::Kind>>,
    attackers_lists: &mut Query<Mut<AttackingList>>,

    mut info: HitInfo,
    attacker_entity: Entity,
    attacker_name: String,

    target_entity: Entity,
    target_vital_stats: &mut Mut<VitalsStats>,

    not_attacking_npc: Query<Entity, (With<npc::Kind>, Without<Attacking>)>,
) {
    if info.miss {
        let miss_message = SystemMessage::new_empty(system_messages::Id::YouHaveMissed);
        commands.trigger_targets(GameServerPacket::from(miss_message), attacker_entity);

        let avoid_message = SystemMessage::new(
            system_messages::Id::YouHaveAvoidedC1SAttack,
            vec![attacker_name.into()],
        );
        commands.trigger_targets(GameServerPacket::from(avoid_message), target_entity);
    } else {
        if info.dmg == 0. {
            //TODO: должен слаться пакет Immune.

            return;
        }

        //TODO: Добавить проверки, на случай цель успела получить целку\камень итд

        // If attacker is NPC, do not damage CP
        let damage_cp = npc.get(attacker_entity).is_err();

        if npc.get(target_entity).is_err() {
            let cur_hp = target_vital_stats.get(&VitalsStat::Hp);

            if info.dmg > cur_hp {
                info.dmg = cur_hp - 10.;
            }
        }

        target_vital_stats.damage(info.dmg, damage_cp);

        // Add damage tracking - this might fail if we just inserted the component
        // but it will be handled in the next frame
        if let Ok(mut attackers_list) = attackers_lists.get_mut(target_entity) {
            attackers_list.damage(attacker_entity, info.dmg as f64);
        }

        if info.crit {
            let critical_system_message =
                SystemMessage::new_empty(system_messages::Id::CriticalHit);
            commands.trigger_targets(
                GameServerPacket::from(critical_system_message),
                attacker_entity,
            );
        }

        send_shield_result_system_message(
            info.shield,
            attacker_entity,
            target_entity,
            commands.reborrow(),
        );

        let you_hit_system_message = SystemMessage::new(
            system_messages::Id::YouHitForS1Damage,
            vec![(info.dmg as u32).into()],
        );
        commands.trigger_targets(
            GameServerPacket::from(you_hit_system_message),
            attacker_entity,
        );

        let you_hitted_system_message = SystemMessage::new(
            system_messages::Id::C1HitYouForS2Damage,
            vec![attacker_name.into(), (info.dmg as u32).into()],
        );
        commands.trigger_targets(
            GameServerPacket::from(you_hitted_system_message),
            target_entity,
        );

        // TODO: Ивент для аи
        // If the target is an NPC that is not already attacking, make it attack back
        if not_attacking_npc.get(target_entity).is_ok() {
            commands
                .entity(target_entity)
                .insert(Attacking(attacker_entity));
        }
    }
}

fn remove_soulshot(mut commands: Commands, weapon_entity: Option<Entity>) {
    if let Some(weapon_entity) = weapon_entity {
        commands.entity(weapon_entity).remove::<Soulshot>();
    }
}

fn calc_hit_info(
    soulshot_used: bool,
    attacker_ref: EntityRef,
    target_ref: EntityRef,
    world: &World,
) -> HitInfo {
    let miss = calc_hit_miss(attacker_ref, target_ref, world);
    let mut crit = false;
    let mut shield = ShieldResult::Failed;
    let mut dmg = 0.;

    if !miss {
        shield = calculate_shield_result(attacker_ref, target_ref, world);

        match shield {
            ShieldResult::Failed | ShieldResult::Succeed => {
                crit = calc_crit(attacker_ref, target_ref, world);
                dmg =
                    calc_p_atk_damage(attacker_ref, target_ref, world, crit, soulshot_used, shield);
            }
            ShieldResult::PerfectBlock => {
                dmg = 1.;
            }
        }
    }

    HitInfo {
        miss,
        crit,
        shield,
        dmg,
    }
}

fn calculate_attack_allowed(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<AttackTimer>), Without<AttackAllowed>>,
    par_commands: ParallelCommands,
) {
    query.par_iter_mut().for_each(|(entity, mut timer)| {
        timer.tick(time.delta());

        if timer.finished() {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).try_insert(AttackAllowed);
            });
            timer.reset();
        }
    });
}

fn setup_attack_timers(
    mut commands: Commands,
    query: Query<(Entity, &AttackStats), Changed<AttackStats>>,
) {
    for (entity, stats) in query.iter() {
        let attack_interval = stats
            .typed::<PAtkSpd>(&AttackStat::PAtkSpd)
            .attack_interval();

        commands
            .entity(entity)
            .try_insert(AttackTimer::new(attack_interval));
    }
}

pub fn in_combat_handler(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<InCombat>)>,
    par_commands: ParallelCommands,
) {
    let delta = time.delta();
    query.par_iter_mut().for_each(|(entity, mut timer)| {
        timer.tick(delta);
        if timer.finished() {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).remove::<InCombat>();
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::serial;
    use game_core::{character::Character, npc::kind::Monster};

    #[test]
    #[serial]
    fn test_attack() {
        let mut app = crate::tests::create_test_app();

        let (character_position, character_entity) = {
            let world = app.world_mut();

            let mut character_query =
                world.query_filtered::<(Entity, Ref<Transform>), With<Character>>();

            let (character_entity, character_transform) = character_query.single(world).unwrap();

            let character_position = character_transform.translation;
            (character_position, character_entity)
        };

        app.update();

        let world = app.world_mut();
        let mut stats_query = world.query_filtered::<Mut<AttackStats>, With<Character>>();

        let mut attack_stats = stats_query.get_mut(world, character_entity).unwrap();

        // Add some PAtk to the character, to kill the mob faster
        attack_stats.insert(AttackStat::PAtk, 30.0);

        let mut mob_query = world.query_filtered::<(Entity, Ref<Transform>), With<Monster>>();

        let (mob_entity, _) = mob_query
            .iter(world)
            .min_by_key(|(_, mob_transform)| {
                character_position.flat_distance(&mob_transform.translation) as i32
            })
            .unwrap();

        if let Ok(mut entity_mut) = world.get_entity_mut(character_entity) {
            entity_mut.insert(Attacking(mob_entity));
        }

        const MAX_ITERATIONS: usize = 50000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            app.update();
            let world = app.world_mut();

            if world.entity(mob_entity).get::<Dead>().is_some() {
                break;
            }

            if iterations >= MAX_ITERATIONS {
                panic!(
                    "Test did not finish after {} iterations. Something is wrong.",
                    MAX_ITERATIONS
                );
            }
        }

        assert!(iterations < MAX_ITERATIONS);
    }
}
