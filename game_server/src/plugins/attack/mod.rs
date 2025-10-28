mod death;
mod packet;

use crate::plugins::stats::{
    CalcCritQuery, CalcShieldQuery, HitMissQuery, PAtkCalcDamageQuery, calc_crit, calc_hit_miss,
    calc_p_atk_damage, calculate_shield_result, send_shield_result_system_message,
};
use avian3d::prelude::*;
use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::SystemParam,
    },
    prelude::*,
};
use game_core::{
    action::wait_kind::Sit,
    active_action::ActiveAction,
    attack::{
        AttackComponentsPlugin, AttackHit, Attacking, AttackingList, ConsumeArrow, Dead, HitInfo,
        Immortal, InCombat, WeaponReuse,
    },
    items::{
        DollSlot, Item, ItemInfo, ItemsDataQuery, Kind, PaperDoll, Soulshot, UniqueItem,
        UpdateType, WeaponAttackParams,
    },
    movement::{LookAt, Movement},
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
            proceed_weapon_reuse.in_set(GameMechanicsSystems::Attacking),
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
    movement: Option<Ref<'a, Movement>>,
    weapon_reuse_active: Has<WeaponReuse>,
    is_sitting: Has<Sit>,
}

#[derive(QueryFilter)]
struct AttackingFilter {
    not_dead: Without<Dead>,
    not_animating: Without<ActiveAction>,
    not_pathfinding: Without<InActionPathfindingTimer>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TargetQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

#[derive(SystemParam)]
struct AttackSystemParams<'w, 's> {
    attacking_objects: Query<'w, 's, AttackingQuery<'static>, AttackingFilter>,
    target_objects: Query<'w, 's, TargetQuery<'static>, (Without<Dead>, With<VitalsStats>)>,
    shot_used: Query<'w, 's, Ref<'static, Soulshot>>,
    items: Query<'w, 's, Entity, With<Item>>,
    items_data_query: ItemsDataQuery<'w>,
    object_id_manager: Res<'w, ObjectIdManager>,
    world_map_query: WorldMapQuery<'w, 's>,
    spatial_query: SpatialQuery<'w, 's>,
    commands: ParallelCommands<'w, 's>,
    hit_calc_query: HitMissQuery<'w, 's>,
    calc_shield_query: CalcShieldQuery<'w, 's>,
    calc_crit_query: CalcCritQuery<'w, 's>,
    p_atk_dmg_query: PAtkCalcDamageQuery<'w, 's>,
}

fn attack_entity(params: AttackSystemParams) -> Result<()> {
    params.attacking_objects.par_iter().for_each(|attacker| {
        if attacker.is_sitting {
            params.commands.command_scope(|mut commands| {
                commands.entity(attacker.entity).remove::<Attacking>();
            });
        }

        match params.target_objects.get(**attacker.target) {
            Ok(aiming_target) => {
                let distance = attacker
                    .transform
                    .translation
                    .flat_distance(&aiming_target.transform.translation);

                let range = attacker.attack_stats.get(AttackStat::PAtkRange);
                if distance <= range {
                    params.commands.command_scope(|mut commands| {
                        commands.trigger_targets(LookAt(aiming_target.entity), attacker.entity);

                        commands
                            .entity(attacker.entity)
                            .remove::<Movement>()
                            .try_insert(InCombat::default());
                    });

                    let weapon_uniq = attacker
                        .paper_doll
                        .as_ref()
                        .and_then(|paper_doll| paper_doll.get(DollSlot::RightHand));

                    let weapon_info = params.items_data_query.item_info_from_uniq(&weapon_uniq);

                    let attack_params =
                        if let Some(Kind::Weapon(weapon)) = weapon_info.map(|v| v.kind()) {
                            weapon.attack_params()
                        } else {
                            WeaponAttackParams::default()
                        };

                    if attack_params.reuse_delay.is_some() && attacker.weapon_reuse_active {
                        return;
                    }

                    if attack_params.is_bow
                        && let Some(paperdoll) = attacker.paper_doll
                    {
                        let arrow_count = paperdoll[DollSlot::LeftHand]
                            .map(|v| v.item().count())
                            .unwrap_or(0);

                        if arrow_count == 0 {
                            params.commands.command_scope(|mut commands| {
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

                        params.commands.command_scope(|mut commands| {
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

                    params.commands.command_scope(|mut commands| {
                        commands
                            .entity(attacker.entity)
                            .try_insert(InCombat::default());
                    });

                    let Ok((attack_packet, attack_hit, attack_interval, weapon_reuse_duration)) =
                        calculate_attack_hit(
                            attacker.entity,
                            aiming_target.entity,
                            range,
                            weapon_uniq,
                            weapon_info,
                            attack_params,
                            &params,
                        )
                    else {
                        return;
                    };

                    if let Some(duration) = weapon_reuse_duration {
                        params.commands.command_scope(|mut commands| {
                            commands.trigger_targets(
                                GameServerPacket::from(SetupGauge::new(
                                    *attacker.object_id,
                                    SetupGaugeColor::Red,
                                    duration,
                                )),
                                attacker.entity,
                            );

                            commands
                                .entity(attacker.entity)
                                .try_insert(WeaponReuse::new(duration));
                        });
                    }

                    params.commands.command_scope(|mut commands| {
                        commands.entity(attacker.entity).try_insert(attack_hit);
                    });

                    params.commands.command_scope(|mut commands| {
                        commands.trigger_targets(
                            ServerPacketBroadcast::new(attack_packet.into()),
                            attacker.entity,
                        );

                        commands
                            .entity(attacker.entity)
                            .try_insert(ActiveAction::new(attack_interval));
                    });
                } else {
                    // Target is out of range, need to chase
                    // Check if already moving to the correct target
                    if let Some(mov) = attacker.movement
                        && mov.is_to_entity()
                        && mov.target() == Some(aiming_target.entity)
                    {
                        return;
                    }

                    let attacker_pos = attacker.transform.translation;
                    let target_pos = aiming_target.transform.translation;

                    let Ok(geodata) = params.world_map_query.region_geodata_from_pos(attacker_pos)
                    else {
                        return;
                    };

                    // Use the same logic as follow plugin - check line of sight
                    let can_move_to = geodata.can_move_to(
                        &WorldMap::vec3_to_geo(attacker_pos),
                        &WorldMap::vec3_to_geo(target_pos),
                    );

                    if can_move_to {
                        // Direct line of sight, use simple movement
                        params.commands.command_scope(|mut commands| {
                            commands
                                .entity(attacker.entity)
                                .try_insert(Movement::to_entity(aiming_target.entity, range));
                        });
                    } else {
                        // No line of sight, use pathfinding
                        params.commands.command_scope(|mut commands| {
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
                params.commands.command_scope(|mut commands| {
                    commands.entity(attacker.entity).remove::<Attacking>();
                });
            }
        }
    });
    Ok(())
}

fn proceed_weapon_reuse(
    time: Res<Time>,
    mut commands: Commands,
    mut characters: Query<(Entity, Mut<WeaponReuse>)>,
) {
    for (character, mut reuse) in characters.iter_mut() {
        if !reuse.proceed_timer(time.delta()) {
            continue;
        }

        commands.entity(character).remove::<WeaponReuse>();
    }
}

fn consume_arrow(
    mut commands: Commands,
    mut characters: Query<(Entity, Mut<PaperDoll>), With<ConsumeArrow>>,
    mut items: Query<(Ref<ObjectId>, Mut<Item>)>,
    object_id_manager: Res<ObjectIdManager>,
) {
    for (char_entity, mut paperdoll) in characters.iter_mut() {
        commands.entity(char_entity).remove::<ConsumeArrow>();

        let Some(paperdoll_left_hand_item) = &mut paperdoll[DollSlot::LeftHand] else {
            continue;
        };

        let left_hand_item_object_id = paperdoll_left_hand_item.object_id();

        let Ok((_, mut left_hand_item)) =
            items.by_object_id_mut(left_hand_item_object_id, object_id_manager.as_ref())
        else {
            warn!("no item with id: {left_hand_item_object_id}");
            continue;
        };

        paperdoll_left_hand_item.item_mut().dec_count();
        left_hand_item.dec_count();

        let unique_item = UniqueItem::new(left_hand_item_object_id, *left_hand_item);

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
    is_immortal: Has<Immortal>,
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
            let should_remove_soulshot = pending_hit.remove_soulshot();
            let weapon_entity = pending_hit.weapon_entity();

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
                    if !pending_dual_hit.set_to_secondary() {
                        commands.entity(attacker.entity).remove::<AttackHit>();
                    }
                }
            }

            if should_remove_soulshot {
                remove_soulshot(commands.reborrow(), weapon_entity);
            }
        }
    }
}

fn process_hit(
    mut commands: Commands,
    npc: Query<Entity, With<npc::Kind>>,
    mut attackers_lists: Query<Mut<AttackingList>>,

    info: HitInfo,
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
        if info.damage == 0. {
            //TODO: должен слаться пакет Immune.
            return;
        }

        //TODO: Добавить проверки, на случай цель успела получить целку\камень итд

        // If attacker is NPC, do not damage CP
        let damage_cp = npc.get(attacker_entity).is_err();

        target
            .vital_stats
            .damage(info.damage, damage_cp, target.is_immortal);

        // Add damage tracking - this might fail if we just inserted the component
        // but it will be handled in the next frame
        if let Ok(mut attackers_list) = attackers_lists.get_mut(target.entity) {
            attackers_list.damage(attacker_entity, info.damage as f64);
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
            vec![(info.damage as u32).into()],
        );
        commands.trigger_targets(
            GameServerPacket::from(you_hit_system_message),
            attacker_entity,
        );

        let you_hitted_system_message = SystemMessage::new(
            system_messages::Id::C1HitYouForS2Damage,
            vec![attacker_name.into(), (info.damage as u32).into()],
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
    attacker: Entity,
    target: Entity,
    weapon_uniq: Option<UniqueItem>,
    weapon_info: Option<&ItemInfo>,
    params: &AttackSystemParams,
) -> Result<HitInfo> {
    // Calculate weapon entity and soulshot usage
    let weapon_entity = weapon_uniq.and_then(|weapon| {
        params
            .items
            .by_object_id(weapon.object_id(), params.object_id_manager.as_ref())
            .ok()
    });

    let soulshot_used = weapon_entity
        .and_then(|v| params.shot_used.get(v).ok())
        .is_some();

    let soulshot_grade = if soulshot_used {
        weapon_info.map(|w| w.grade().shot_grade())
    } else {
        None
    };

    let miss = calc_hit_miss(attacker, target, &params.hit_calc_query)?;
    let mut crit = false;
    let mut shield = ShieldResult::Failed;
    let mut damage = 0.;

    if !miss {
        shield = calculate_shield_result(attacker, target, &params.calc_shield_query);

        match shield {
            ShieldResult::Failed | ShieldResult::Succeed => {
                crit = calc_crit(attacker, target, &params.calc_crit_query);
                damage = calc_p_atk_damage(
                    attacker,
                    target,
                    crit,
                    soulshot_used,
                    shield,
                    &params.p_atk_dmg_query,
                );
            }
            ShieldResult::PerfectBlock => {
                damage = 1.;
            }
        }
    }

    Ok(HitInfo {
        ss_grade: soulshot_grade,
        miss,
        crit,
        shield,
        damage,
    })
}

fn calculate_attack_hit(
    attacker: Entity,
    aiming_target: Entity,
    range: f32,
    weapon_uniq: Option<UniqueItem>,
    weapon_info: Option<&ItemInfo>,
    attack_params: WeaponAttackParams,
    params: &AttackSystemParams,
) -> Result<(Attack, AttackHit, Duration, Option<Duration>)> {
    let (attacker_stats, attacker_position, attacker_oid) = {
        let attacker_params = params.attacking_objects.get(attacker)?;
        (
            attacker_params.attack_stats,
            attacker_params.transform.translation,
            attacker_params.object_id,
        )
    };

    let target = params.target_objects.get(aiming_target)?;
    let aiming_target_position = target.transform.translation;

    let mut attack_packet = Attack::new(*attacker_oid, attacker_position, aiming_target_position);

    let attacker_attack_speed = attacker_stats.typed::<PAtkSpd>(AttackStat::PAtkSpd);
    let attack_interval = attacker_attack_speed.attack_interval();

    //TODO: найти точную формулу, проверить должна ли пропадать красная полоска при смене оружия
    let weapon_reuse_duration = attack_params.reuse_delay.map(|weapon_reuse_delay| {
        let atck_speed: u32 = attacker_attack_speed.into();
        Duration::from_millis(
            //TODO: должна быть базовая скорость атаки. 300 для игроков, 333 для мобов
            weapon_reuse_delay as u64 * 300 / atck_speed as u64,
        ) + attack_interval
    });

    let mut max_targets_count = attacker_stats.get(AttackStat::PAtkMaxTargetsCount).round() as u32;

    // Calculate weapon entity for later use
    let weapon_entity = weapon_uniq.and_then(|weapon| {
        params
            .items
            .by_object_id(weapon.object_id(), params.object_id_manager.as_ref())
            .ok()
    });

    if max_targets_count > 1 {
        let mut hits = vec![];

        let hit_info = calc_hit_info(attacker, aiming_target, weapon_uniq, weapon_info, params)?;

        let mut all_missed = hit_info.miss;

        attack_packet.add_hit(*target.object_id, hit_info);

        hits.push((aiming_target, hit_info));

        max_targets_count -= 1;

        let attack_angle = attacker_stats.get(AttackStat::PAtkWidth).round();

        let attack_vector = attacker_position - aiming_target_position;

        // Use SpatialQuery to find nearby entities within weapon range
        let query_sphere = Collider::sphere(range);
        let filter =
            SpatialQueryFilter::default().with_excluded_entities([attacker, aiming_target]);

        let nearby_entities = params.spatial_query.shape_intersections(
            &query_sphere,
            attacker_position,
            Quat::IDENTITY,
            &filter,
        );

        for next_target_entity in nearby_entities {
            let Ok(next_target) = params.target_objects.get(next_target_entity) else {
                continue;
            };

            let next_target_vector = attacker_position - next_target.transform.translation;

            //TODO: в калькуляторе переводить в PI все градусы
            let angle_in_degrees = attack_vector.angle_between(next_target_vector) * 180. / PI;

            if angle_in_degrees > attack_angle {
                continue;
            }

            //TODO: нужна проверка на враждебность
            let hit_info = calc_hit_info(
                attacker,
                next_target.entity,
                weapon_uniq,
                weapon_info,
                params,
            )?;

            all_missed &= hit_info.miss;

            attack_packet.add_hit(*next_target.object_id, hit_info);

            hits.push((next_target.entity, hit_info));

            if max_targets_count == 1 {
                break;
            } else {
                max_targets_count -= 1;
            }
        }

        let attack_hit = AttackHit::new_multi(
            attack_interval.mul_f32(attack_params.primary_attack_delay_multiplier),
            weapon_entity,
            hits,
            all_missed,
        );

        Ok((
            attack_packet,
            attack_hit,
            attack_interval,
            weapon_reuse_duration,
        ))
    } else {
        let hit_info = calc_hit_info(attacker, aiming_target, weapon_uniq, weapon_info, params)?;

        attack_packet.add_hit(*target.object_id, hit_info);

        let attack_hit = if let Some(second_attack_interval_multiplier) =
            attack_params.secondary_attack_delay_multiplier
        {
            let second_hit_info =
                calc_hit_info(attacker, aiming_target, weapon_uniq, weapon_info, params)?;

            attack_packet.add_hit(*target.object_id, second_hit_info);

            AttackHit::new_dual(
                aiming_target,
                weapon_entity,
                attack_interval.mul_f32(attack_params.primary_attack_delay_multiplier),
                hit_info,
                attack_interval.mul_f32(second_attack_interval_multiplier),
                second_hit_info,
                hit_info.miss & second_hit_info.miss,
            )
        } else {
            AttackHit::new_common(
                aiming_target,
                attack_interval.mul_f32(attack_params.primary_attack_delay_multiplier),
                hit_info,
                weapon_entity,
            )
        };

        Ok((
            attack_packet,
            attack_hit,
            attack_interval,
            weapon_reuse_duration,
        ))
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
