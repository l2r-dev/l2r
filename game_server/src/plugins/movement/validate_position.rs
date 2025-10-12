use bevy::{log, prelude::*};
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
    stats::Movable,
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
const MAX_DISTANCE_THRESHOLD: f32 = 512.0;
const MAX_DISTANCE_IN_WATER: f32 = MAX_DISTANCE_THRESHOLD * 2.0;

fn validate_position_handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,

    mut movable_objects: Query<
        (
            Ref<ObjectId>,
            Mut<Transform>,
            Ref<Movable>,
            Option<Ref<TeleportInProgress>>,
        ),
        With<Movable>,
    >,
    world_map_query: WorldMapQuery,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::ValidatePosition(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let (object_id, mut transform, movable, teleport_request) =
            movable_objects.get_mut(character_entity)?;
        if teleport_request.is_some() {
            return Ok(());
        }

        let mut server_pos = transform.translation;
        let client_pos = packet.location;
        let region_id = RegionId::from(server_pos);

        let geodata = world_map_query.region_geodata(region_id).ok();

        if let Some(geodata) = geodata
            && let Some(geodata_height) = geodata.nearest_height(&WorldMap::vec3_to_geo(server_pos))
        {
            let geodata_height = geodata_height as f32;
            if server_pos.y > geodata_height && (geodata_height - server_pos.y).abs() > 150.0 {
                log::trace!(
                    "Entity {:?} is too far from the ground. Skipping position validation.",
                    character_entity
                );
                transform.translation = client_pos;
            } else if (server_pos.y - geodata_height).abs() > 16.0 {
                log::trace!(
                    "Adjusting height from {} to {}",
                    server_pos.y,
                    geodata_height
                );
                transform.translation.y = geodata_height;
                server_pos.y = geodata_height;
            }
        }

        // Calculate the distance in each axis
        let delta_x = (server_pos.x - client_pos.x).abs();
        let delta_y = (server_pos.y - client_pos.y).abs();
        let delta_z = (server_pos.z - client_pos.z).abs();

        if delta_x > HORIZONTAL_THRESHOLD
            || delta_y > VERTICAL_THRESHOLD
            || delta_z > HORIZONTAL_THRESHOLD
        {
            let distance = delta_x + delta_y + delta_z;

            if !movable.is_flying() && !movable.in_water() {
                log::trace!(
                    "<{:?}> unsync ({:?}): server: {:?}, client: {:?}",
                    object_id,
                    distance,
                    server_pos,
                    client_pos,
                );
            }

            let is_falling = delta_z > VERTICAL_THRESHOLD && server_pos.y < client_pos.y;
            match (movable.in_water(), is_falling, distance) {
                // In water
                (true, _, dist) if dist < MAX_DISTANCE_IN_WATER => {
                    transform.translation = client_pos;
                    return Ok(());
                }
                // Entity is not in water and falling
                (false, true, _) => {
                    log::debug!(
                        "Entity {:?} is falling. Skipping position validation.",
                        character_entity
                    );
                    transform.translation = client_pos;
                }
                // Distance within threshold
                (_, _, dist) if dist < MAX_DISTANCE_THRESHOLD => {
                    transform.translation = client_pos;
                }
                _ => commands.trigger_targets(
                    GameServerPacket::from(ValidateLocation::new(*object_id, *transform)),
                    character_entity,
                ),
            }
        }
    }
    Ok(())
}
