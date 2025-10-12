use bevy::{log, prelude::*};

mod cannot_move_anymore;
mod follow;
mod move_backward_to_location;
mod swimming;
mod validate_position;
mod walk_run;

use bevy::ecs::{
    query::{QueryData, QueryFilter},
    system::{ParallelCommands, SystemParam},
};
use game_core::{
    action::{pickup::PickupRequest, wait_kind::Sit},
    animation::{Animation, AnimationFinished},
    attack::{AttackHit, Dead},
    movement::{
        ArrivedAtWaypoint, LookAt, MoveStep, MoveTarget, MoveToEntity, MovementComponentsPlugin,
        SendStopMove,
    },
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{ActionFail, GameServerPacket, MoveToLocation, MoveToPawn, StopMove},
    },
    object_id::ObjectId,
    stats::Movable,
};
use map::{WorldMap, WorldMapQuery};
use spatial::FlatDistance;
use state::GameServerStateSystems;

// 3.0 is okay for normal char speed up to 600, if speed is more, it may not arrive at waypoints
// Don't want to calculate it, cause game speed limit is 300.
const ARRIVAL_DISTANCE: f32 = 3.0;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MovementComponentsPlugin);

        app.add_plugins(move_backward_to_location::MoveBackwardToLocationPlugin)
            .add_plugins(validate_position::ValidatePositionPlugin)
            .add_plugins(cannot_move_anymore::CannotMoveAnymorePlugin)
            .add_plugins(follow::FollowPlugin)
            .add_plugins(walk_run::WalkRunPlugin)
            .add_plugins(swimming::SwimmingPlugin);

        app.add_systems(FixedUpdate, move_entity.in_set(GameServerStateSystems::Run));
        app.add_systems(
            Update,
            send_move_to_location.in_set(GameServerStateSystems::Run),
        );
        app.add_systems(
            Update,
            send_move_to_pawn_packet.in_set(GameServerStateSystems::Run),
        );

        app.add_observer(handle_waypoint_arrival)
            .add_observer(handle_movement_step)
            .add_observer(send_stop_move_packet)
            .add_observer(look_at_target)
            .add_observer(move_to_target_after_animation)
            .add_observer(move_to_entity_after_animation);
    }
}

#[derive(QueryFilter)]
struct MoveFilter {
    not_animating: Without<Animation>,
    not_dead: Without<Dead>,
    not_sitting: Without<Sit>,
}

/// Entities changed in movement state
#[derive(QueryFilter)]
struct MovementChangeFilter {
    move_target_changed: Changed<MoveTarget>,
    not_animating: Without<Animation>,
    not_dead: Without<Dead>,
}

// Entities changed in move-to-entity state
#[derive(QueryFilter)]
struct MoveToEntityChangeFilter {
    move_to_entity_changed: Changed<MoveToEntity>,
    not_animating: Without<Animation>,
    not_dead: Without<Dead>,
    not_sitting: Without<Sit>,
}

#[derive(QueryData)]
struct MoveQuery<'a> {
    entity: Entity,
    transform: Ref<'a, Transform>,
    move_target: Option<Ref<'a, MoveTarget>>,
    move_to_entity: Option<Ref<'a, MoveToEntity>>,
}

#[derive(SystemParam)]
struct MovementQueries<'w, 's> {
    transforms: Query<'w, 's, Ref<'static, Transform>>,
    movable_query: Query<'w, 's, Ref<'static, Movable>>,
}

// Move a bit closer to avoid arrival distance issues
const MOVE_TO_ENTITY_ADJUSTMENT: f32 = 3.0;

fn move_entity(
    queries: MovementQueries,
    query: Query<MoveQuery, MoveFilter>,
    par_commands: ParallelCommands,
) {
    query.par_iter().for_each(|moving| {
        if let Some(move_to) = moving.move_to_entity {
            if let Ok(target_transform) = queries.transforms.get(move_to.target) {
                let distance = moving
                    .transform
                    .translation
                    .flat_distance(&target_transform.translation)
                    + MOVE_TO_ENTITY_ADJUSTMENT;

                if distance > move_to.range {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(MoveStep, moving.entity);
                    });
                }
            } else {
                par_commands.command_scope(|mut commands| {
                    commands.entity(moving.entity).remove::<MoveToEntity>();
                });
            }
        } else if let Some(move_target) = moving.move_target {
            if move_target.is_empty() {
                return;
            }

            if let Some(current_wp) = move_target.front() {
                let distance = if let Ok(movable) = queries.movable_query.get(moving.entity)
                    && (movable.in_water() || movable.is_flying())
                {
                    moving.transform.translation.distance(*current_wp.target())
                } else {
                    moving
                        .transform
                        .translation
                        .flat_distance(current_wp.target())
                };

                if distance <= ARRIVAL_DISTANCE {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(ArrivedAtWaypoint, moving.entity);
                    });
                } else {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(MoveStep, moving.entity);
                    });
                }
            }
        }
    });
}

#[derive(QueryData)]
#[query_data(mutable)]
struct WaypointArrivalQuery<'a> {
    movable: Mut<'a, Movable>,
    move_target: Mut<'a, MoveTarget>,
    transform: Ref<'a, Transform>,
}

fn handle_waypoint_arrival(
    arrived: Trigger<ArrivedAtWaypoint>,
    mut commands: Commands,
    mut query: Query<WaypointArrivalQuery>,
) {
    let entity = arrived.target();
    if let Ok(mut arriving) = query.get_mut(entity) {
        arriving.move_target.pop_front();
        arriving.movable.reset_steps();

        if let Some(next_wp) = arriving.move_target.front_mut() {
            if *next_wp.origin() != arriving.transform.translation {
                next_wp.set_origin(arriving.transform.translation);
            }
        } else {
            commands.entity(entity).remove::<MoveTarget>();
        }
    }
}

#[derive(SystemParam)]
struct MovementStepQueries<'w, 's> {
    time: Res<'w, Time<Fixed>>,
    world_map_query: WorldMapQuery<'w, 's>,
    transforms: Query<'w, 's, (Ref<'static, ObjectId>, Mut<'static, Transform>)>,
    movables: Query<'w, 's, MovablesQuery<'static>, MovablesFilter>,
}

#[derive(QueryFilter)]
struct MovablesFilter {
    not_dead: Without<Dead>,
    not_sitting: Without<Sit>,
    not_animating: Without<Animation>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovablesQuery<'a> {
    movable: Mut<'a, Movable>,
    move_target: Option<Mut<'a, MoveTarget>>,
    move_to_entity: Option<Ref<'a, MoveToEntity>>,
}

/// Moves entity one step toward its target, handling collisions and height.
fn handle_movement_step(
    step: Trigger<MoveStep>,
    mut commands: Commands,
    mut queries: MovementStepQueries,
) {
    let entity = step.target();
    if let Ok(mut movables) = queries.movables.get_mut(entity) {
        let moving_to_entity = movables.move_to_entity.is_some();
        let mut target_pos = if let Some(move_to) = movables.move_to_entity {
            if let Ok((_, target_transform)) = queries.transforms.get(move_to.target) {
                target_transform.translation
            } else {
                commands.entity(entity).remove::<MoveToEntity>();
                return;
            }
        } else if let Some(move_target) = movables.move_target.as_ref() {
            if let Some(next_wp) = move_target.front() {
                *next_wp.target()
            } else {
                return; // No waypoints; should not happen due to move_entity logic
            }
        } else {
            return; // Neither component present; should not happen
        };

        let Ok((object_id, mut transform)) = queries.transforms.get_mut(entity) else {
            return;
        };
        let mut current_pos = transform.translation;

        let geodata = queries
            .world_map_query
            .region_geodata_from_pos(current_pos)
            .ok();

        if !movables.movable.in_water() || movables.movable.is_flying() {
            current_pos.y = match geodata
                .and_then(|geodata| geodata.nearest_height(&WorldMap::vec3_to_geo(current_pos)))
            {
                Some(height) => height as f32,
                None => target_pos.y,
            };

            target_pos.y = current_pos.y;
        } else {
            current_pos.y = transform.translation.y;
        }

        transform.look_at(target_pos, Vec3::Y);

        let delta_time = queries.time.delta_secs();
        let speed = movables.movable.speed() as f32;
        let move_distance = speed * delta_time;
        let direction = (target_pos - current_pos).normalize();
        let move_step = direction * move_distance;
        let new_position = current_pos + move_step;

        movables.movable.step();

        let can_move = geodata.is_none_or(|geodata| {
            if movables.movable.in_water() || movables.movable.is_flying() {
                true
            } else {
                geodata.can_move_to(
                    &WorldMap::vec3_to_geo(current_pos),
                    &WorldMap::vec3_to_geo(new_position),
                )
            }
        });

        if !can_move {
            if movables.move_target.is_some() {
                if let Some(mut move_target) = movables.move_target {
                    move_target.clear();
                }
            } else if moving_to_entity {
                commands.entity(entity).remove::<MoveToEntity>();
            }

            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            let stop_move = StopMove::new(*object_id, *transform);
            commands.trigger_targets(ServerPacketBroadcast::new(stop_move.into()), entity);
            return;
        }

        transform.translation = new_position;

        if let Some(move_target) = movables.move_target.as_ref()
            && let Some(wp) = move_target.front()
        {
            let wp_pos = wp.target();

            let distance = if movables.movable.in_water() || movables.movable.is_flying() {
                current_pos.distance(*wp_pos)
            } else {
                current_pos.flat_distance(wp_pos)
            };

            if distance < ARRIVAL_DISTANCE {
                commands.trigger_targets(ArrivedAtWaypoint, entity);
            }
        }
    }
}

#[derive(QueryData)]
struct MovementTargetQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    move_target: Ref<'a, MoveTarget>,
    pickup_request: Option<Ref<'a, PickupRequest>>,
}

fn send_move_to_location(
    movable_objects: Query<MovementTargetQuery, MovementChangeFilter>,
    mut commands: Commands,
) {
    for movement_data in &movable_objects {
        log::trace!(
            "Move target changed by {:?}",
            movement_data.move_target.changed_by()
        );
        type ConflictingComponents = (MoveToEntity, AttackHit);

        if let Some(wp) = movement_data.move_target.front() {
            commands
                .entity(movement_data.entity)
                .remove::<ConflictingComponents>();

            let packet = MoveToLocation::new(*movement_data.object_id, *wp.origin(), *wp.target());
            commands.trigger_targets(
                ServerPacketBroadcast::new(packet.into()),
                movement_data.entity,
            );
        }
    }
}

fn move_to_target_after_animation(
    _animation_finished: Trigger<AnimationFinished>,
    movable_objects: Query<MovementTargetQuery, MovementChangeFilter>,
    commands: Commands,
) {
    send_move_to_location(movable_objects, commands);
}

#[derive(QueryData)]
struct MoveToEntityQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    move_to_entity: Ref<'a, MoveToEntity>,
    transform: Ref<'a, Transform>,
}

#[derive(QueryData)]
pub struct TransformObjectQuery<'a> {
    pub object_id: Ref<'a, ObjectId>,
    pub transform: Ref<'a, Transform>,
}

fn move_to_entity_after_animation(
    _animation_finished: Trigger<AnimationFinished>,
    move_to_entity: Query<MoveToEntityQuery, MoveToEntityChangeFilter>,
    targets: Query<TransformObjectQuery>,
    commands: Commands,
) {
    send_move_to_pawn_packet(commands, move_to_entity, targets);
}

fn send_move_to_pawn_packet(
    mut commands: Commands,
    move_to_entity: Query<MoveToEntityQuery, MoveToEntityChangeFilter>,
    targets: Query<TransformObjectQuery>,
) {
    for move_data in &move_to_entity {
        if let Ok(target_data) = targets.get(move_data.move_to_entity.target) {
            commands.entity(move_data.entity).remove::<MoveTarget>();
            let packet = MoveToPawn::new(
                *move_data.object_id,
                *target_data.object_id,
                move_data.transform.translation,
                target_data.transform.translation,
                move_data.move_to_entity.range as u32,
            );
            commands.trigger_targets(ServerPacketBroadcast::new(packet.into()), move_data.entity);
        } else {
            commands.entity(move_data.entity).remove::<MoveToEntity>();
        }
    }
}

pub fn send_stop_move_packet(
    stop: Trigger<SendStopMove>,
    transforms: Query<TransformObjectQuery>,
    mut commands: Commands,
) {
    let entity = stop.target();
    if let Ok(transform_data) = transforms.get(entity) {
        let packet = StopMove::new(*transform_data.object_id, *transform_data.transform);
        commands.trigger_targets(ServerPacketBroadcast::new(packet.into()), entity);
    }
}

fn look_at_target(look_at: Trigger<LookAt>, mut transforms: Query<Mut<Transform>>) -> Result<()> {
    let entity = look_at.target();
    let target = look_at.event().0;

    let target_transform = *transforms.get(target)?;
    let mut requester_transform = transforms.get_mut(entity)?;

    requester_transform.look_at(target_transform.translation, Vec3::Y);

    Ok(())
}
