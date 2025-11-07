use avian3d::prelude::*;
use bevy::prelude::*;
use game_core::{
    attack::Immortal,
    character::Character,
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{
            BroadcastCharInfo, ChangeMoveType, GameServerPacket, SendUserInfo, SetupGauge,
            SetupGaugeColor,
        },
    },
    npc,
    object_id::ObjectId,
    stats::*,
};
use map::{Water, Zone, ZoneKind};
use state::StatKindSystems;
use std::time::Duration;

pub struct SwimmingPlugin;
impl Plugin for SwimmingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_water_zone_collisions,
                handle_breath.in_set(StatKindSystems::Other),
                restore_breath_out_of_water.in_set(StatKindSystems::Other),
            ),
        );
    }
}

fn handle_water_zone_collisions(
    mut collision_started_events: EventReader<CollisionStarted>,
    mut collision_ended_events: EventReader<CollisionEnded>,
    mut commands: Commands,
    mut char_movables: Query<(Ref<ObjectId>, Mut<Movable>, Ref<OtherStats>), With<Character>>,
    mut npc_movables: Query<(Ref<ObjectId>, Mut<Movable>), (With<npc::Kind>, Without<Character>)>,
    water_zones: Query<Ref<Zone>, With<Water>>,
) {
    for CollisionStarted(entity1, entity2) in collision_started_events.read() {
        if let Some((movable_entity, zone_entity)) =
            determine_entities(*entity1, *entity2, &water_zones)
            && let Ok(zone) = water_zones.get(zone_entity)
            && matches!(zone.kind(), ZoneKind::Water)
        {
            if let Ok((object_id, mut movable, other_stats)) = char_movables.get_mut(movable_entity)
            {
                set_swimming_mode(
                    commands.reborrow(),
                    movable.reborrow(),
                    *object_id,
                    movable_entity,
                );
                commands.trigger_targets(SendUserInfo, movable_entity);
                commands.trigger_targets(BroadcastCharInfo, movable_entity);

                let current_breath = other_stats.get(OtherStat::Breath);
                let max_breath = other_stats.get(OtherStat::BreathMax);
                send_breath_gauge(
                    commands.reborrow(),
                    movable_entity,
                    *object_id,
                    current_breath,
                    max_breath,
                );
            } else if let Ok((object_id, mut movable)) = npc_movables.get_mut(movable_entity) {
                set_swimming_mode(
                    commands.reborrow(),
                    movable.reborrow(),
                    *object_id,
                    movable_entity,
                );
            }
        }
    }

    for CollisionEnded(entity1, entity2) in collision_ended_events.read() {
        if let Some((movable_entity, zone_entity)) =
            determine_entities(*entity1, *entity2, &water_zones)
            && let Ok(zone) = water_zones.get(zone_entity)
            && matches!(zone.kind(), ZoneKind::Water)
        {
            if let Ok((object_id, mut movable, _)) = char_movables.get_mut(movable_entity) {
                unset_swimming_mode(
                    commands.reborrow(),
                    movable.reborrow(),
                    *object_id,
                    movable_entity,
                );
                commands.trigger_targets(SendUserInfo, movable_entity);
                commands.trigger_targets(BroadcastCharInfo, movable_entity);

                // Hide breath gauge when exiting water (set to 0 duration)
                commands.trigger_targets(
                    GameServerPacket::from(SetupGauge::new(
                        *object_id,
                        SetupGaugeColor::Blue,
                        Duration::from_secs(0),
                    )),
                    movable_entity,
                );
            } else if let Ok((object_id, mut movable)) = npc_movables.get_mut(movable_entity) {
                unset_swimming_mode(
                    commands.reborrow(),
                    movable.reborrow(),
                    *object_id,
                    movable_entity,
                );
            }
        }
    }
}

/// Determine which entity is the movable entity and which is the zone
fn determine_entities(
    entity1: Entity,
    entity2: Entity,
    water_zones: &Query<Ref<Zone>, With<Water>>,
) -> Option<(Entity, Entity)> {
    if water_zones.get(entity1).is_ok() {
        Some((entity2, entity1))
    } else if water_zones.get(entity2).is_ok() {
        Some((entity1, entity2))
    } else {
        None
    }
}

fn set_swimming_mode(
    mut commands: Commands,
    mut movable: Mut<Movable>,
    object_id: ObjectId,
    entity: Entity,
) {
    match movable.move_type() {
        MovementStat::Walk => {
            movable.set_move_type(MovementStat::Swim);
            broadcast_movement_change(commands.reborrow(), object_id, MovementStat::Swim, entity);
        }
        MovementStat::Run => {
            movable.set_move_type(MovementStat::FastSwim);
            broadcast_movement_change(
                commands.reborrow(),
                object_id,
                MovementStat::FastSwim,
                entity,
            );
        }
        _ => {
            // Already swimming or flying, no change needed
        }
    }
}

fn unset_swimming_mode(
    mut commands: Commands,
    mut movable: Mut<Movable>,
    object_id: ObjectId,
    entity: Entity,
) {
    // Only change from swimming to ground movement
    match movable.move_type() {
        MovementStat::Swim => {
            movable.set_move_type(MovementStat::Walk);
            broadcast_movement_change(commands.reborrow(), object_id, MovementStat::Walk, entity);
        }
        MovementStat::FastSwim => {
            movable.set_move_type(MovementStat::Run);
            broadcast_movement_change(commands.reborrow(), object_id, MovementStat::Run, entity);
        }
        _ => {
            // Not swimming, no change needed
        }
    }
}

fn broadcast_movement_change(
    mut commands: Commands,
    object_id: ObjectId,
    new_move_type: MovementStat,
    entity: Entity,
) {
    commands.trigger_targets(
        ServerPacketBroadcast::new(ChangeMoveType::new(object_id, new_move_type).into()),
        entity,
    );
}

const BREATH_DRAIN_PERIOD: f32 = 1.0;
const OUT_OF_BREATH_DAMAGE_PERCENT: f32 = 1.0;

fn handle_breath(
    mut commands: Commands,
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut swimming_entities: Query<
        (
            Entity,
            Ref<ObjectId>,
            Ref<Movable>,
            Mut<OtherStats>,
            Mut<VitalsStats>,
            Has<Immortal>,
        ),
        With<Character>,
    >,
) {
    let time_spent = time.elapsed_secs() - *last_time;
    if time_spent >= BREATH_DRAIN_PERIOD {
        *last_time = time.elapsed_secs();

        for (entity, object_id, movable, mut other_stats, mut vitals_stats, is_immortal) in
            swimming_entities.iter_mut()
        {
            if movable.in_water() {
                let current_breath = other_stats.get(OtherStat::Breath);
                let max_breath = other_stats.get(OtherStat::BreathMax);

                if current_breath > 0.0 {
                    let new_breath = (current_breath - 1.0).max(0.0);
                    other_stats.insert(OtherStat::Breath, new_breath);
                    send_breath_gauge(
                        commands.reborrow(),
                        entity,
                        *object_id,
                        new_breath,
                        max_breath,
                    );
                } else {
                    vitals_stats.percent_stat_damage(
                        VitalsStat::Hp,
                        OUT_OF_BREATH_DAMAGE_PERCENT,
                        is_immortal,
                    );
                }
            }
        }
    }
}

fn send_breath_gauge(
    mut commands: Commands,
    entity: Entity,
    object_id: ObjectId,
    current_breath: f32,
    max_breath: f32,
) {
    let current_duration = Duration::from_secs(current_breath as u64);
    let total_duration = Duration::from_secs(max_breath as u64);

    commands.trigger_targets(
        GameServerPacket::from(SetupGauge::new_with_current(
            object_id,
            SetupGaugeColor::Blue,
            current_duration,
            total_duration,
        )),
        entity,
    );
}

const BREATH_RESTORE_PERIOD: f32 = 1.0;
const BREATH_RESTORE_PERCENT: f32 = 0.05;

fn restore_breath_out_of_water(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut characters: Query<(&Movable, &mut OtherStats), With<Character>>,
) {
    let time_spent = time.elapsed_secs() - *last_time;
    if time_spent >= BREATH_RESTORE_PERIOD {
        *last_time = time.elapsed_secs();

        for (movable, mut other_stats) in characters.iter_mut() {
            if !movable.in_water() {
                let current_breath = other_stats.get(OtherStat::Breath);
                let max_breath = other_stats.get(OtherStat::BreathMax);

                // Restore % per second
                if current_breath < max_breath {
                    let new_breath =
                        (current_breath + (max_breath * BREATH_RESTORE_PERCENT)).min(max_breath);
                    other_stats.insert(OtherStat::Breath, new_breath);
                }
            }
        }
    }
}
