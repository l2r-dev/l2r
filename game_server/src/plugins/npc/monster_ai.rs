use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        relationship::Relationship,
    },
    prelude::*,
};
use game_core::{
    attack::InCombat,
    custom_hierarchy::DespawnChildOf,
    movement::Movement,
    npc::{NpcAiComponentsPlugin, RandomWalkingTimer, kind::Monster},
    spawner::Spawner,
};
use map::{WorldMapQuery, id::RegionId};
use rand::Rng;
use spatial::WayPoint;

const MAX_DISTANCE_FROM_PARENT: f32 = 800.0; // Maximum distance NPCs can wander from their spawn point
const NORMAL_WALK_RADIUS: i32 = 100; // Normal random walk radius when close to parent
const RETURN_WALK_RADIUS: f32 = 0.3; // Radius around parent when returning (30% of max distance)

pub struct NpcAiPlugin;
impl Plugin for NpcAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NpcAiComponentsPlugin);

        app.add_systems(Update, (random_walking_setup, random_walking_around));
    }
}

#[derive(QueryFilter)]
struct WalkingSetupFilter {
    monster: With<Monster>,
    no_timer: Without<RandomWalkingTimer>,
}

fn random_walking_setup(mut commands: Commands, non_walkers: Query<Entity, WalkingSetupFilter>) {
    for entity in non_walkers.iter() {
        commands
            .entity(entity)
            .try_insert(RandomWalkingTimer::default());
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RandomWalkingQuery<'a> {
    entity: Entity,
    transform: &'a Transform,
    timer: &'a mut RandomWalkingTimer,
    parent: Option<&'a DespawnChildOf>,
}

#[derive(QueryFilter)]
struct RandomWalkingFilter {
    move_target: Without<Movement>,
    not_in_combat: Without<InCombat>,
}

fn random_walking_around(
    mut commands: Commands,
    time: Res<Time>,
    mut random_walkers: Query<RandomWalkingQuery, RandomWalkingFilter>,
    map_query: WorldMapQuery,
    parents: Query<&Transform, With<Spawner>>,
) {
    for mut walker in random_walkers.iter_mut() {
        walker.timer.tick(time.delta());
        if walker.timer.finished() {
            let current_pos = walker.transform.translation;
            let region_id = RegionId::from(current_pos);

            let parent_pos = if let Some(parent_ref) = walker.parent {
                if let Ok(transform) = parents.get(parent_ref.get()) {
                    transform.translation
                } else {
                    current_pos
                }
            } else {
                current_pos
            };

            let distance_from_parent = current_pos.distance(parent_pos);
            let (target_center, search_radius) = if distance_from_parent > MAX_DISTANCE_FROM_PARENT
            {
                // Too far from parent, move back towards parent
                (
                    parent_pos,
                    (MAX_DISTANCE_FROM_PARENT * RETURN_WALK_RADIUS) as i32,
                )
            } else {
                // Close enough to parent, random walk around current position
                let random_radius = rand::thread_rng().gen_range(10..NORMAL_WALK_RADIUS);
                (current_pos, random_radius)
            };

            let geodata = map_query.inner.region_geodata(region_id).ok();

            if let Some(geodata) = geodata
                && let Some(random_point) =
                    geodata.random_point_in_radius_vec3(target_center, search_radius)
            {
                // Ensure the new point doesn't exceed max distance from parent
                let final_point = if parent_pos.distance(random_point) <= MAX_DISTANCE_FROM_PARENT {
                    random_point
                } else {
                    // Clamp the point to be within max distance
                    let direction = (random_point - parent_pos).normalize();
                    parent_pos + direction * (MAX_DISTANCE_FROM_PARENT * 0.9)
                };
                commands
                    .entity(walker.entity)
                    .try_insert(Movement::to_waypoint(WayPoint::new(
                        current_pos,
                        final_point,
                    )));
            }
            walker
                .timer
                .set_duration(RandomWalkingTimer::random_duration());
        }
    }
}
