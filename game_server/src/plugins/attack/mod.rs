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
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    animation::{Animation, AnimationTimer},
    attack::{
        AttackAllowed, AttackComponentsPlugin, AttackHit, AttackTimer, Attacking, AttackingList,
        ConsumeArrow, Dead, HitInfo, InCombat,
    },
    items::{
        DollSlot, Grade, Item, ItemsDataQuery, Kind, PaperDoll, Soulshot, UniqueItem, UpdateType,
        WeaponAttackParams,
    },
    movement::{LookAt, MoveToEntity},
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{
            Attack, GameServerPacket, InventoryUpdate, SetupGauge, SetupGaugeColor, SystemMessage,
        },
    },
    npc,
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId, QueryByObjectIdMut},
    path_finding::{InActionPathfindingTimer, VisibilityCheckRequest},
    stats::*,
};
use map::{WorldMap, WorldMapQuery};
use smallvec::smallvec;
use spatial::FlatDistance;
use state::GameMechanicsSystems;
use std::{f32::consts::PI, time::Duration};
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
            consume_arrow.in_set(GameMechanicsSystems::Attacking),
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
struct TargetQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

fn attack_entity(
    attacking_objects: Query<AttackingQuery, AttackingFilter>,
    target_objects: Query<TargetQuery, (Without<Dead>, With<VitalsStats>)>,
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

                let range = attacker.attack_stats.get(AttackStat::PAtkRange);
                if distance <= range {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(LookAt(aiming_target.entity), attacker.entity);
                        // Remove the AttackAllowed component to ensure the entity waits for
                        // the next allowed attack time
                        commands
                            .entity(attacker.entity)
                            .remove::<AttackAllowed>()
                            .remove::<MoveToEntity>();

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

                    let attacker_attack_speed =
                        attacker.attack_stats.typed::<PAtkSpd>(&AttackStat::PAtkSpd);

                    let attack_interval = attacker_attack_speed.attack_interval();

                    let weapon_uniq = attacker
                        .paper_doll
                        .as_ref()
                        .and_then(|paper_doll| paper_doll.get(DollSlot::RightHand));

                    let weapon_info = items_data_query.item_info_from_uniq(&weapon_uniq);

                    let (
                        interval_multiplier,
                        second_attack_interval_multiplier,
                        weapon_reuse_delay,
                        is_bow,
                    ) = if let Some(Kind::Weapon(weapon)) = weapon_info.map(|v| v.kind()) {
                        let WeaponAttackParams {
                            is_bow,
                            reuse_delay,
                            primary_attack_delay_multiplier,
                            secondary_attack_delay_multiplier,
                        } = weapon.attack_params();

                        (
                            primary_attack_delay_multiplier,
                            secondary_attack_delay_multiplier,
                            reuse_delay,
                            is_bow,
                        )
                    } else {
                        (0.5, None, None, false)
                    };

                    if is_bow && let Some(paperdoll) = attacker.paper_doll {
                        let arrow_count = paperdoll[DollSlot::LeftHand]
                            .map(|v| v.item().count())
                            .unwrap_or(0);

                        if arrow_count == 0 {
                            par_commands.command_scope(|mut commands| {
                                commands.trigger_targets(
                                    GameServerPacket::from(SystemMessage::new(
                                        system_messages::Id::YouHaveRunOutOfArrows,
                                        vec![],
                                    )),
                                    attacker.entity,
                                );

                                commands.entity(attacker.entity).remove::<Attacking>();
                            });

                            return;
                        }

                        par_commands.command_scope(|mut commands| {
                            commands.entity(attacker.entity).try_insert(ConsumeArrow);

                            commands.trigger_targets(
                                GameServerPacket::from(SystemMessage::new(
                                    system_messages::Id::YouCarefullyNockAnArrow,
                                    vec![],
                                )),
                                attacker.entity,
                            );
                        });
                    }

                    par_commands.command_scope(|mut commands| {
                        commands
                            .entity(attacker.entity)
                            .try_insert(InCombat::default());
                    });

                    let weapon_entity = weapon_uniq.and_then(|weapon| {
                        items
                            .by_object_id(weapon.object_id(), object_id_manager.as_ref())
                            .ok()
                    });

                    let soulshot_used = weapon_entity.and_then(|v| shot_used.get(v).ok()).is_some();

                    let soulshot_grade = if let Some(w) = weapon_info {
                        w.grade()
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

                    if let Some(weapon_reuse_delay) = weapon_reuse_delay {
                        let atck_speed: u32 = attacker_attack_speed.into();

                        let delay = weapon_reuse_delay as f32 / atck_speed as f32 * 0.5;

                        par_commands.command_scope(|mut commands| {
                            commands.trigger_targets(
                                GameServerPacket::from(SetupGauge::new(
                                    *attacker.object_id,
                                    SetupGaugeColor::Red,
                                    Duration::from_secs_f32(delay),
                                )),
                                attacker.entity,
                            );
                        });
                    }

                    let attack_hit = if max_targets_count > 1 {
                        let mut hits = vec![];
                      
                        let attack_interval = attacker
                            .attack_stats
                            .typed::<PAtkSpd>(AttackStat::PAtkSpd)
                            .attack_interval();

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

                        let attack_angle = attacker_ref
                            .get::<AttackStats>()
                            .map(|s| s.get(&AttackStat::PAtkWidth))
                            .unwrap_or_default()
                            .round();

                        //TODO: Хак! Сейчас если персонаж убегает от цели и нажать атаку, он все еще будет смотреть от нее
                        // видимо нужно поменять порядок систем
                        let attack_vector =
                            attacker.transform.translation - aiming_target.transform.translation;

                        //TODO: Хак! aoe_targets_query это перебор по всем сущностям вообще. Нужна структура, чтобы быстро находить кто вокруг
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

                            let next_target_vector =
                                attacker.transform.translation - next_target.transform.translation;

                            //TODO: в калькуляторе переводить в PI все градусы
                            let angle_in_degrees =
                                attack_vector.angle_between(next_target_vector) * 180. / PI;

                            if angle_in_degrees > attack_angle {
                                continue;
                            }

                            //TODO: нужна проверка на враждебность

                            let hit_info =
                                calc_hit_info(soulshot_used, attacker_ref, next_target_ref, world);

                            attack_info.add_hit(
                                hit_info.dmg as u32,
                                *next_target.object_id,
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

fn consume_arrow(
    mut commands: Commands,
    mut characters: Query<(Entity, Mut<PaperDoll>), With<ConsumeArrow>>,
    mut items: Query<(Ref<ObjectId>, Mut<Item>)>,
    object_id_manager: Res<ObjectIdManager>,
) {
    for (char_entity, mut paperdoll) in characters.iter_mut() {
        commands.entity(char_entity).remove::<ConsumeArrow>();

        let Some(left_hand_item_uniq) = &mut paperdoll[DollSlot::LeftHand] else {
            continue;
        };

        let left_hand_item_object_id = left_hand_item_uniq.object_id();

        let Ok((_, mut left_hand_item)) =
            items.by_object_id_mut(left_hand_item_object_id, object_id_manager.as_ref())
        else {
            continue;
        };

        left_hand_item_uniq.item_mut().dec_count();
        left_hand_item.dec_count();

        let item = *left_hand_item;
        commands.spawn_task(move || async move {
            item.update_count_in_database(left_hand_item_object_id)
                .await
        });

        let unique_item = UniqueItem::new(left_hand_item_object_id, item);

        commands.trigger_targets(
            GameServerPacket::from(InventoryUpdate::new(
                smallvec![unique_item],
                UpdateType::Modify,
            )),
            char_entity,
        );
    }
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
            match &mut *pending_hit {
                AttackHit::AttackCommonHit(pending_hit) => {
                    if let Ok(mut target) = targets.get_mut(pending_hit.target()) {
                        let info = pending_hit.hit();

                        process_hit(
                            commands.reborrow(),
                            npc,
                            attackers_lists.reborrow(),
                            info,
                            attacker.entity,
                            attacker.name.to_string(),
                            &mut target,
                            not_attacking_npc,
                        );
                    }

                    remove_soulshot(commands.reborrow(), pending_hit.weapon_entity());

                    commands.entity(attacker.entity).remove::<AttackHit>();
                }

                AttackHit::AttackMultiHit(pending_hits) => {
                    for (target, info) in pending_hits.hits() {
                        if let Ok(mut target) = targets.get_mut(*target) {
                            process_hit(
                                commands.reborrow(),
                                npc,
                                attackers_lists.reborrow(),
                                *info,
                                attacker.entity,
                                attacker.name.to_string(),
                                &mut target,
                                not_attacking_npc,
                            );
                        }
                    }

                    remove_soulshot(commands.reborrow(), pending_hits.weapon_entity());

                    commands.entity(attacker.entity).remove::<AttackHit>();
                }

                AttackHit::AttackDualHit(pending_dual_hit) => {
                    if let Ok(mut target) = targets.get_mut(pending_dual_hit.target()) {
                        let info = pending_dual_hit.hit();

                        process_hit(
                            commands.reborrow(),
                            npc,
                            attackers_lists.reborrow(),
                            info,
                            attacker.entity,
                            attacker.name.to_string(),
                            &mut target,
                            not_attacking_npc,
                        );
                    }

                    if pending_dual_hit.set_to_secondary() {
                        remove_soulshot(commands.reborrow(), pending_dual_hit.weapon_entity());
                    } else {
                        commands.entity(attacker.entity).remove::<AttackHit>();
                    }
                }
            }
        }
    }
}

fn process_hit(
    mut commands: Commands,
    npc: Query<Entity, With<npc::Kind>>,
    mut attackers_lists: Query<Mut<AttackingList>>,

    mut info: HitInfo,
    attacker_entity: Entity,
    attacker_name: String,
    target: &mut ProcessAttackHitsTargetQueryItem,

    not_attacking_npc: Query<Entity, (With<npc::Kind>, Without<Attacking>)>,
) {
    if info.miss {
        let miss_message = SystemMessage::new_empty(system_messages::Id::YouHaveMissed);
        commands.trigger_targets(GameServerPacket::from(miss_message), attacker_entity);

        let avoid_message = SystemMessage::new(
            system_messages::Id::YouHaveAvoidedC1SAttack,
            vec![attacker_name.into()],
        );
        commands.trigger_targets(GameServerPacket::from(avoid_message), target.entity);
    } else {
        if info.dmg == 0. {
            //TODO: должен слаться пакет Immune.

            return;
        }

        //TODO: Добавить проверки, на случай цель успела получить целку\камень итд

        // If attacker is NPC, do not damage CP
        let damage_cp = npc.get(attacker_entity).is_err();

        if npc.get(target.entity).is_err() {
            let cur_hp = target.vital_stats.get(&VitalsStat::Hp);

            //TODO: Хак чтобы постоянно не умирать, переделать на Undying компонент
            if info.dmg > cur_hp {
                info.dmg = cur_hp - 10.;
            }
        }

        target.vital_stats.damage(info.dmg, damage_cp);

        // Add damage tracking - this might fail if we just inserted the component
        // but it will be handled in the next frame
        if let Ok(mut attackers_list) = attackers_lists.get_mut(target.entity) {
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
            target.entity,
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
            target.entity,
        );

        // TODO: Ивент для аи
        // If the target is an NPC that is not already attacking, make it attack back
        if not_attacking_npc.get(target.entity).is_ok() {
            commands
                .entity(target.entity)
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
            .typed::<PAtkSpd>(AttackStat::PAtkSpd)
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
