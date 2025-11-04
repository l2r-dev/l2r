use bevy::prelude::*;
use game_core::{active_action::ActiveAction, movement::Falling, stats::Movable};
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
    par_cmd: ParallelCommands,
    query: Query<(Entity, Ref<Transform>, Ref<Movable>), Without<Falling>>,
    map_query: WorldMapQuery,
) -> Result<()> {
    query.par_iter().for_each(|(entity, transform, movable)| {
        if movable.is_flying() || movable.in_water() {
            return;
        }

        let Ok(geodata) = map_query
            .inner
            .region_geodata_from_pos(transform.translation)
        else {
            return;
        };

        if let Some(geodata_height) =
            geodata.nearest_height(WorldMap::vec3_to_geo(transform.translation))
        {
            let height = geodata_height as f32;
            let distance_to_ground = transform.translation.y - height;

            if distance_to_ground > FALL_DETECTION_DISTANCE {
                let fall_duration = Duration::from_secs_f32(distance_to_ground / FALLING_SPEED);
                par_cmd.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .insert((Falling, ActiveAction::new(fall_duration)));
                });
            }
        }
    });
    Ok(())
}

fn apply_gravity(
    par_cmd: ParallelCommands,
    mut query: Query<(Entity, Mut<Transform>, Ref<Movable>), With<Falling>>,
    time: Res<Time<Fixed>>,
    map_query: WorldMapQuery,
) {
    let delta_time = time.delta_secs();

    query
        .par_iter_mut()
        .for_each(|(entity, mut transform, movable)| {
            if movable.is_flying() || movable.in_water() {
                par_cmd.command_scope(|mut commands| {
                    commands.entity(entity).remove::<Falling>();
                });
                return;
            }

            let Ok(geodata) = map_query
                .inner
                .region_geodata_from_pos(transform.translation)
            else {
                par_cmd.command_scope(|mut commands| {
                    commands.entity(entity).remove::<Falling>();
                });
                return;
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
                    par_cmd.command_scope(|mut commands| {
                        commands.entity(entity).remove::<(Falling, ActiveAction)>();
                    });
                } else {
                    transform.translation.y = new_y;
                }
            } else {
                par_cmd.command_scope(|mut commands| {
                    commands.entity(entity).remove::<(Falling, ActiveAction)>();
                });
            }
        });
}
