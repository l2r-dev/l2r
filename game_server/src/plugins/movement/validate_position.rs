use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, ValidateLocation},
        },
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
    stats::{LastKnownPosition, Movable},
    teleport::TeleportInProgress,
};
use map::{WorldMap, WorldMapQuery, id::RegionId};

pub(crate) struct ValidatePositionPlugin;
impl Plugin for ValidatePositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(validate_position_handle);
    }
}

const HORIZONTAL_THRESHOLD: f32 = 128.0;
const VERTICAL_THRESHOLD: f32 = 128.0;
const MAX_DISTANCE_THRESHOLD: f32 = HORIZONTAL_THRESHOLD + VERTICAL_THRESHOLD;
const MAX_DISTANCE_IN_WATER: f32 = MAX_DISTANCE_THRESHOLD * 2.0;
const MAX_DISTANCE_FLYING: f32 = MAX_DISTANCE_THRESHOLD * 2.0;
const GEODATA_HEIGHT_TOLERANCE: f32 = 16.0;
const MAX_GROUND_SNAP_DISTANCE: f32 = 50.0;

const SPEED_CHECK_MIN_TIME: f32 = 0.1;
const SPEED_TOLERANCE_MULTIPLIER: f32 = 1.5;
const SPEED_VIOLATION_THRESHOLD: f32 = 2.0;

// Using squared distances for performance
const HORIZONTAL_THRESHOLD_SQ: f32 = HORIZONTAL_THRESHOLD * HORIZONTAL_THRESHOLD;
const VERTICAL_THRESHOLD_SQ: f32 = VERTICAL_THRESHOLD * VERTICAL_THRESHOLD;
const MAX_DISTANCE_IN_WATER_SQ: f32 = MAX_DISTANCE_IN_WATER * MAX_DISTANCE_IN_WATER;
const MAX_DISTANCE_FLYING_SQ: f32 = MAX_DISTANCE_FLYING * MAX_DISTANCE_FLYING;

/// Validates player position reported by client against server-side position.
fn validate_position_handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    time: Res<Time>,

    mut movable_objects: Query<
        (
            Ref<ObjectId>,
            Mut<Transform>,
            Ref<Movable>,
            Mut<LastKnownPosition>,
            Has<TeleportInProgress>,
        ),
        With<Movable>,
    >,
    world_map_query: WorldMapQuery,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::ValidatePosition(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let (object_id, mut transform, movable, mut known_pos, teleport_request) =
            movable_objects.get_mut(character_entity)?;

        // Skip validation if teleport is in progress
        if teleport_request {
            return Ok(());
        }

        let current_time = time.elapsed_secs_f64();
        let client_pos = packet.location;
        let region_id = RegionId::from(transform.translation);

        let is_flying = movable.is_flying();
        let is_in_water = movable.in_water();
        let is_exiting_water = movable.exiting_water();

        let geodata = world_map_query.region_geodata(region_id)?;

        if !is_in_water
            && let Some(geodata_height) =
                geodata.nearest_height(&WorldMap::vec3_to_geo(transform.translation))
        {
            let geodata_height = geodata_height as f32;
            let distance_to_ground = (transform.translation.y - geodata_height).abs();

            if is_exiting_water && distance_to_ground > MAX_GROUND_SNAP_DISTANCE {
                transform.translation = known_pos.position;
                commands.trigger_targets(
                    GameServerPacket::from(ValidateLocation::new(*object_id, *transform)),
                    character_entity,
                );
                return Ok(());
            } else if is_flying {
                // Flying entities can't go underground - enforce minimum height
                if transform.translation.y < geodata_height {
                    transform.translation.y = geodata_height;
                }
            } else if distance_to_ground < MAX_GROUND_SNAP_DISTANCE {
                // Ground entities snap to geodata height only if close enough
                let height_diff = (transform.translation.y - geodata_height).abs();
                if height_diff > GEODATA_HEIGHT_TOLERANCE {
                    transform.translation.y = geodata_height;
                }
            }
        }

        // Validate client movement speed by comparing actual distance traveled
        // with maximum allowed distance based on movement speed and elapsed time
        let time_elapsed = (current_time - known_pos.timestamp) as f32;
        if time_elapsed > SPEED_CHECK_MIN_TIME {
            let move_speed = movable.speed() as f32;
            let client_movement_distance = client_pos.distance(known_pos.position);

            let max_allowed_distance = move_speed * time_elapsed;
            let max_allowed_with_tolerance = max_allowed_distance * SPEED_TOLERANCE_MULTIPLIER;

            let speed_ratio = client_movement_distance / max_allowed_distance;
            if client_movement_distance > max_allowed_with_tolerance
                && speed_ratio > SPEED_VIOLATION_THRESHOLD
            {
                commands.trigger_targets(
                    GameServerPacket::from(ValidateLocation::new(*object_id, *transform)),
                    character_entity,
                );
                known_pos.position = transform.translation;
                known_pos.timestamp = current_time;
                return Ok(());
            }
        }

        let delta = transform.translation - client_pos;
        let horizontal_dist_sq = delta.x * delta.x + delta.z * delta.z;
        let vertical_dist_sq = delta.y * delta.y;
        let total_dist_sq = transform.translation.distance_squared(client_pos);

        if horizontal_dist_sq > HORIZONTAL_THRESHOLD_SQ || vertical_dist_sq > VERTICAL_THRESHOLD_SQ
        {
            let distance = total_dist_sq.sqrt();

            if !is_flying && !is_in_water {
                debug!(
                    "<{:?}> Position desync - server: {:?}, client: {:?}, distance: {:.1}",
                    object_id, transform.translation, client_pos, distance
                );
            }

            // Check if entity is falling (client position is higher than server)
            let is_falling =
                vertical_dist_sq > VERTICAL_THRESHOLD_SQ && client_pos.y > transform.translation.y;

            // When flying or in water, allow larger position corrections
            // TODO: We already checked for maximum speed over time above
            // TODO: So it's pretty safe to trust client position here, but maybe we can improve it further?
            match (is_flying, is_in_water, total_dist_sq) {
                // Flying
                (true, _, dist_sq) if dist_sq < MAX_DISTANCE_FLYING_SQ => {
                    transform.translation = client_pos;
                }
                // Swimming
                (_, true, dist_sq) if dist_sq < MAX_DISTANCE_IN_WATER_SQ => {
                    transform.translation = client_pos;
                }
                // Entity is falling
                (false, false, _) if is_falling => {
                    transform.translation.y = client_pos.y;
                }
                // All other cases - server position takes priority
                _ => {
                    commands.trigger_targets(
                        GameServerPacket::from(ValidateLocation::new(*object_id, *transform)),
                        character_entity,
                    );
                }
            }
        }

        // Update last known position after all validation
        known_pos.position = transform.translation;
        known_pos.timestamp = current_time;
    }
    Ok(())
}
