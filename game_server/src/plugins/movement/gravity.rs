use bevy::prelude::*;
use game_core::{
    active_action::ActiveAction, character::Character, movement::Falling, stats::Movable,
};
use map::{WorldMap, WorldMapQuery};
use state::GameServerStateSystems;
use std::time::Duration;

pub struct GravityPlugin;
impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (detect_falling, apply_gravity)
                .chain()
                .in_set(GameServerStateSystems::Run),
        );
    }
}

const FALLING_SPEED: f32 = 900.0;
const FALL_DETECTION_DISTANCE: f32 = 16.0;

fn detect_falling(
    mut commands: Commands,
    query: Query<(Entity, Ref<Transform>, Ref<Movable>), (With<Character>, Without<Falling>)>,
    map_query: WorldMapQuery,
) -> Result<()> {
    for (entity, transform, movable) in query.iter() {
        if movable.is_flying() || movable.in_water() {
            continue;
        }

        let Ok(geodata) = map_query
            .inner
            .region_geodata_from_pos(transform.translation)
        else {
            continue;
        };

        if let Some(geodata_height) =
            geodata.nearest_height(WorldMap::vec3_to_geo(transform.translation))
        {
            let height = geodata_height as f32;
            let distance_to_ground = transform.translation.y - height;

            if distance_to_ground > FALL_DETECTION_DISTANCE {
                let fall_duration = Duration::from_secs_f32(distance_to_ground / FALLING_SPEED);
                commands
                    .entity(entity)
                    .insert((Falling, ActiveAction::new(fall_duration)));
            }
        }
    }
    Ok(())
}

fn apply_gravity(
    mut commands: Commands,
    mut query: Query<(Entity, Mut<Transform>, Ref<Movable>), With<Falling>>,
    time: Res<Time<Fixed>>,
    map_query: WorldMapQuery,
) {
    let delta_time = time.delta_secs();

    for (entity, mut transform, movable) in query.iter_mut() {
        if movable.is_flying() || movable.in_water() {
            commands.entity(entity).remove::<Falling>();
            continue;
        }

        let Ok(geodata) = map_query
            .inner
            .region_geodata_from_pos(transform.translation)
        else {
            commands.entity(entity).remove::<Falling>();
            continue;
        };

        if let Some(geodata_height) =
            geodata.nearest_height(WorldMap::vec3_to_geo(transform.translation))
        {
            let height = geodata_height as f32;

            let fall_distance = FALLING_SPEED * delta_time;
            let new_y = transform.translation.y - fall_distance;

            // Check if we've reached the ground
            if new_y <= height {
                transform.translation.y = height;
                commands.entity(entity).remove::<(Falling, ActiveAction)>();
            } else {
                transform.translation.y = new_y;
            }
        } else {
            commands.entity(entity).remove::<(Falling, ActiveAction)>();
        }
    }
}
