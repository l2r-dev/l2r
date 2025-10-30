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
    action::wait_kind::Sit,
    active_action::{ActionFinished, ActiveAction},
    attack::{AttackHit, Dead},
    movement::{
        ArrivedAtWaypoint, LookAt, MoveStep, Movement, MovementComponentsPlugin, SendStopMove,
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
const MAX_GROUND_SNAP_DISTANCE: f32 = 32.0;

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

        app.add_systems(
            FixedUpdate,
            handle_movement.in_set(GameServerStateSystems::Run),
        );
        app.add_systems(
            Update,
            send_movement_packets.in_set(GameServerStateSystems::Run),
        );

        app.add_observer(handle_waypoint_arrival)
            .add_observer(handle_movement_step)
            .add_observer(send_stop_move_packet)
            .add_observer(look_at_target)
            .add_observer(movement_after_action_finished);
    }
}

#[derive(QueryFilter)]
struct MoveFilter {
    not_in_action: Without<ActiveAction>,
}

/// Entities changed in movement state
#[derive(QueryFilter)]
struct MovementChangeFilter {
    movement_changed: Changed<Movement>,
    not_in_action: Without<ActiveAction>,
}

#[derive(SystemParam)]
struct MovementQueries<'w, 's> {
    transforms: Query<'w, 's, Ref<'static, Transform>>,
    movable_query: Query<'w, 's, Ref<'static, Movable>>,
}

// Move a bit closer to avoid arrival distance issues
const MOVE_TO_ENTITY_ADJUSTMENT: f32 = 3.0;

#[derive(QueryData)]
struct MovementQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    movement: Ref<'a, Movement>,
    transform: Ref<'a, Transform>,
    is_sitting: Has<Sit>,
    is_dead: Has<Dead>,
}

fn handle_movement(
    queries: MovementQueries,
    query: Query<MovementQuery, MoveFilter>,
    par_commands: ParallelCommands,
) {
    query
        .par_iter()
        .for_each(|moving| match moving.movement.as_ref() {
            Movement::ToEntity { target, range } => {
                if let Ok(target_transform) = queries.transforms.get(*target) {
                    let distance = moving
                        .transform
                        .translation
                        .flat_distance(&target_transform.translation)
                        + MOVE_TO_ENTITY_ADJUSTMENT;

                    if distance > *range {
                        par_commands.command_scope(|mut commands| {
                            commands.trigger_targets(MoveStep, moving.entity);
                        });
                    }
                } else {
                    par_commands.command_scope(|mut commands| {
                        commands.entity(moving.entity).remove::<Movement>();
                    });
                }
            }
            Movement::ToLocation { waypoints } => {
                if waypoints.is_empty() {
                    return;
                }

                if let Some(current_wp) = waypoints.front() {
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
    movement: Mut<'a, Movement>,
    transform: Ref<'a, Transform>,
    is_sitting: Has<Sit>,
    is_dead: Has<Dead>,
}

fn handle_waypoint_arrival(
    arrived: Trigger<ArrivedAtWaypoint>,
    mut commands: Commands,
    mut query: Query<WaypointArrivalQuery>,
) {
    let entity = arrived.target();
    if let Ok(mut arriving) = query.get_mut(entity)
        && let Some(waypoints) = arriving.movement.waypoints_mut()
    {
        waypoints.pop_front();
        arriving.movable.reset_steps();

        if let Some(next_wp) = waypoints.front_mut() {
            if *next_wp.origin() != arriving.transform.translation {
                next_wp.set_origin(arriving.transform.translation);
            }
        } else {
            commands.entity(entity).remove::<Movement>();
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
    not_animating: Without<ActiveAction>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovablesQuery<'a> {
    movable: Mut<'a, Movable>,
    movement: Option<Mut<'a, Movement>>,
    is_sitting: Has<Sit>,
    is_dead: Has<Dead>,
}

/// Moves entity one step toward its target, handling collisions and height.
fn handle_movement_step(
    step: Trigger<MoveStep>,
    mut commands: Commands,
    mut queries: MovementStepQueries,
) {
    let entity = step.target();
    if let Ok(mut movables) = queries.movables.get_mut(entity) {
        let Some(ref mut movement) = movables.movement else {
            return; // No movement component
        };

        let mut target_pos = match movement.as_ref() {
            Movement::ToEntity { target, .. } => {
                if let Ok((_, target_transform)) = queries.transforms.get(*target) {
                    target_transform.translation
                } else {
                    commands.entity(entity).remove::<Movement>();
                    return;
                }
            }
            Movement::ToLocation { waypoints } => {
                if let Some(next_wp) = waypoints.front() {
                    *next_wp.target()
                } else {
                    return; // No waypoints
                }
            }
        };

        let Ok((object_id, mut transform)) = queries.transforms.get_mut(entity) else {
            return;
        };
        let mut current_pos = transform.translation;

        let geodata = queries
            .world_map_query
            .region_geodata_from_pos(current_pos)
            .ok();

        if !movables.movable.in_water()
            && !movables.movable.is_flying()
            && let Some(geodata_height) =
                geodata.and_then(|gd| gd.nearest_height(&WorldMap::vec3_to_geo(current_pos)))
        {
            let height = geodata_height as f32;
            let distance_to_ground = (current_pos.y - height).abs();

            if distance_to_ground < MAX_GROUND_SNAP_DISTANCE {
                current_pos.y = height;
                target_pos.y = height;
            }
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
            } else if movables.movable.exiting_water() {
                // Prevent movement through terrain when transitioning from water to land
                if let Some(geodata_height) =
                    geodata.nearest_height(&WorldMap::vec3_to_geo(current_pos))
                {
                    let distance_to_ground = (current_pos.y - geodata_height as f32).abs();
                    distance_to_ground < MAX_GROUND_SNAP_DISTANCE
                        && geodata.can_move_to(
                            &WorldMap::vec3_to_geo(current_pos),
                            &WorldMap::vec3_to_geo(new_position),
                        )
                } else {
                    // No geodata - allow movement
                    true
                }
            } else {
                geodata.can_move_to(
                    &WorldMap::vec3_to_geo(current_pos),
                    &WorldMap::vec3_to_geo(new_position),
                )
            }
        });

        if !can_move {
            commands.entity(entity).remove::<Movement>();
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            let stop_move = StopMove::new(*object_id, *transform);
            commands.trigger_targets(ServerPacketBroadcast::new(stop_move.into()), entity);
            return;
        }

        transform.translation = new_position;

        if let Some(Movement::ToLocation { waypoints }) = movables.movement.as_deref()
            && let Some(wp) = waypoints.front()
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

fn send_movement_packets(
    movable_objects: Query<MovementQuery, MovementChangeFilter>,
    targets: Query<TransformObjectQuery>,
    mut commands: Commands,
) {
    for movement_data in &movable_objects {
        log::trace!(
            "Movement changed by {:?}",
            movement_data.movement.changed_by()
        );

        commands.entity(movement_data.entity).remove::<AttackHit>();

        match movement_data.movement.as_ref() {
            Movement::ToLocation { waypoints } => {
                if let Some(wp) = waypoints.front() {
                    let packet =
                        MoveToLocation::new(*movement_data.object_id, *wp.origin(), *wp.target());
                    commands.trigger_targets(
                        ServerPacketBroadcast::new(packet.into()),
                        movement_data.entity,
                    );
                }
            }
            Movement::ToEntity { target, range } => {
                if let Ok(target_data) = targets.get(*target) {
                    let packet = MoveToPawn::new(
                        *movement_data.object_id,
                        *target_data.object_id,
                        movement_data.transform.translation,
                        target_data.transform.translation,
                        *range as u32,
                    );
                    commands.trigger_targets(
                        ServerPacketBroadcast::new(packet.into()),
                        movement_data.entity,
                    );
                } else {
                    commands.entity(movement_data.entity).remove::<Movement>();
                }
            }
        }
    }
}

fn movement_after_action_finished(
    _action_finished: Trigger<ActionFinished>,
    movable_objects: Query<MovementQuery, MovementChangeFilter>,
    targets: Query<TransformObjectQuery>,
    commands: Commands,
) {
    send_movement_packets(movable_objects, targets, commands);
}

#[derive(QueryData)]
pub struct TransformObjectQuery<'a> {
    pub object_id: Ref<'a, ObjectId>,
    pub transform: Ref<'a, Transform>,
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
