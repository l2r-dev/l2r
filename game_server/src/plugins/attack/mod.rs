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
        Dead, InCombat,
    },
    items::{DollSlot, Grade, Item, ItemsDataQuery, PaperDoll, Soulshot},
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
use spatial::FlatDistance;
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
    enemy_objects: Query<EnemyQuery, Without<Dead>>,
    names: Query<Ref<Name>>,
    world: &World,
    shot_used: Query<Ref<Soulshot>>,
    par_commands: ParallelCommands,
    items: Query<Entity, With<Item>>,
    items_data_query: ItemsDataQuery,
    object_id_manager: Res<ObjectIdManager>,
    world_map_query: WorldMapQuery,
) -> Result<()> {
    attacking_objects.par_iter().for_each(|attacker| {
        match enemy_objects.get(**attacker.target) {
            Ok(enemy) => {
                let distance = attacker
                    .transform
                    .translation
                    .flat_distance(&enemy.transform.translation);

                let range = attacker.attack_stats.get(&AttackStat::PAtkRange);
                if distance <= range {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(LookAt(enemy.entity), attacker.entity);
                        // Remove the AttackAllowed component to ensure the entity waits for
                        // the next allowed attack time
                        commands
                            .entity(attacker.entity)
                            .remove::<AttackAllowed>()
                            .try_insert(InCombat::default());
                        commands
                            .entity(enemy.entity)
                            .try_insert(InCombat::default());
                    });

                    let weapon = attacker
                        .paper_doll
                        .and_then(|paper_doll| paper_doll.get(DollSlot::RightHand));

                    let weapon_entity = weapon.and_then(|weapon| {
                        items
                            .by_object_id(weapon.object_id(), object_id_manager.as_ref())
                            .ok()
                    });

                    let (shot_kind, shot_grade) = if let Some(weapon) = weapon {
                        if let Ok(weapon_item_info) =
                            items_data_query.get_item_info(weapon.item().id())
                        {
                            if let Some(weapon_entity) = weapon_entity {
                                if let Ok(shot) = shot_used.get(weapon_entity) {
                                    (Some(*shot), weapon_item_info.grade())
                                } else {
                                    (None, Grade::None)
                                }
                            } else {
                                (None, Grade::None)
                            }
                        } else {
                            (None, Grade::None)
                        }
                    } else {
                        (None, Grade::None)
                    };

                    let Ok(attacker_ref) = world.get_entity(attacker.entity) else {
                        return;
                    };
                    let Ok(enemy_ref) = world.get_entity(enemy.entity) else {
                        return;
                    };

                    let miss = calc_hit_miss(attacker_ref, enemy_ref, world);
                    let crit = calc_crit(attacker_ref, enemy_ref, world);

                    let mut attack = Attack::new(
                        *attacker.object_id,
                        attacker.transform.translation,
                        enemy.transform.translation,
                    );

                    if !miss {
                        let shield_result = calculate_shield_result(attacker_ref, enemy_ref, world);

                        let damage = calc_p_atk_damage(
                            attacker_ref,
                            enemy_ref,
                            world,
                            crit,
                            shot_kind.is_some(),
                            shield_result,
                        );

                        attack.add_hit(
                            damage as u32,
                            *enemy.object_id,
                            miss,
                            crit,
                            0,
                            shot_kind.is_some(),
                            shot_grade,
                        );

                        par_commands.command_scope(|mut commands| {
                            commands.trigger_targets(
                                ServerPacketBroadcast::new(attack.into()),
                                attacker.entity,
                            );

                            let attack_interval = attacker
                                .attack_stats
                                .typed::<PAtkSpd>(&AttackStat::PAtkSpd)
                                .attack_interval();

                            // TODO: Must depend on weapon type
                            let hit_delay = attack_interval / 2;

                            send_shield_result_system_message(
                                shield_result,
                                enemy.entity,
                                commands.reborrow(),
                            );

                            commands
                                .entity(attacker.entity)
                                .try_insert(AnimationTimer::new(attack_interval));

                            commands.entity(attacker.entity).try_insert(AttackHit::new(
                                enemy.entity,
                                damage,
                                crit,
                                hit_delay,
                                weapon_entity,
                            ));
                        });
                    } else {
                        attack.add_hit(
                            0, // No damage on miss
                            *enemy.object_id,
                            miss,
                            crit,
                            0,
                            shot_kind.is_some(),
                            shot_grade,
                        );

                        par_commands.command_scope(|mut commands| {
                            commands.trigger_targets(
                                ServerPacketBroadcast::new(attack.into()),
                                attacker.entity,
                            );

                            let miss_message =
                                SystemMessage::new_empty(system_messages::Id::YouHaveMissed);
                            commands.trigger_targets(
                                GameServerPacket::from(miss_message),
                                attacker.entity,
                            );

                            let attacker_name = names.get(attacker.entity).unwrap().to_string();
                            let avoid_message = SystemMessage::new(
                                system_messages::Id::YouHaveAvoidedC1SAttack,
                                vec![attacker_name.to_string().into()],
                            );
                            commands.trigger_targets(
                                GameServerPacket::from(avoid_message),
                                enemy.entity,
                            );
                        });
                    }

                    par_commands.command_scope(|mut commands| {
                        commands.entity(attacker.entity).remove::<MoveToEntity>();
                    });
                } else {
                    // Target is out of range, need to chase
                    // Check if already moving to the correct target
                    if let Some(move_to) = attacker.move_to
                        && move_to.target == enemy.entity
                    {
                        return;
                    }

                    let attacker_pos = attacker.transform.translation;
                    let target_pos = enemy.transform.translation;

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
                                target: enemy.entity,
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

fn process_attack_hits(
    mut commands: Commands,
    mut pending_attacks: Query<(Entity, Ref<Name>, Mut<AttackHit>)>,
    time: Res<Time>,
    mut vitals: Query<Mut<VitalsStats>>,
    not_attacking_npcs: Query<Entity, (With<npc::Kind>, Without<Attacking>)>,
    npcs: Query<Entity, With<npc::Kind>>,
    mut attackers_lists: Query<Mut<AttackingList>>,
) {
    for (attacker_entity, name, mut pending) in pending_attacks.iter_mut() {
        pending.timer_mut().tick(time.delta());
        if pending.timer().finished() {
            if let Ok(mut vitals_stats) = vitals.get_mut(pending.target()) {
                // If attacker is NPC, do not damage CP
                let damage_cp = npcs.get(attacker_entity).is_err();

                vitals_stats.damage(pending.damage(), damage_cp);

                // Add damage tracking - this might fail if we just inserted the component
                // but it will be handled in the next frame
                if let Ok(mut attackers_list) = attackers_lists.get_mut(pending.target()) {
                    attackers_list.damage(attacker_entity, pending.damage() as f64);
                }

                if pending.critical() {
                    let critical_system_message =
                        SystemMessage::new_empty(system_messages::Id::CriticalHit);
                    commands.trigger_targets(
                        GameServerPacket::from(critical_system_message),
                        attacker_entity,
                    );
                }

                let you_hit_system_message = SystemMessage::new(
                    system_messages::Id::YouHitForS1Damage,
                    vec![(pending.damage() as u32).into()],
                );
                commands.trigger_targets(
                    GameServerPacket::from(you_hit_system_message),
                    attacker_entity,
                );

                let you_hitted_system_message = SystemMessage::new(
                    system_messages::Id::C1HitYouForS2Damage,
                    vec![name.to_string().into(), (pending.damage() as u32).into()],
                );
                commands.trigger_targets(
                    GameServerPacket::from(you_hitted_system_message),
                    pending.target(),
                );
            }

            commands.entity(attacker_entity).remove::<AttackHit>();

            // Remove soulshot from weapon after successful hit
            if let Some(weapon_entity) = pending.weapon_entity() {
                commands.entity(weapon_entity).remove::<Soulshot>();
            }

            // If the target is an NPC that is not already attacking, make it attack back
            if not_attacking_npcs.get(pending.target()).is_ok() {
                commands
                    .entity(pending.target())
                    .insert(Attacking(attacker_entity));
            }
        }
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
