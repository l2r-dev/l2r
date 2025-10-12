use bevy::{ecs::system::ParallelCommands, log, prelude::*};
use game_core::{
    movement::MoveTarget,
    network::packets::server::{DropItem, GameServerPacket},
    path_finding::{
        DropDebugItem, InActionPathfindingTimer, PathFindingComponentsPlugin, PathFindingRequest,
        PathFindingResult, VisibilityCheckRequest, VisibilityCheckResult, WAYPOINTS_CAPACITY,
    },
    stats::Movable,
};
use map::{RegionGeoData, WorldMap, WorldMapQuery, id::RegionId};
use smallvec::SmallVec;
use spatial::{GeoVec3, WayPoint};

const STRAIGHT_WEIGHT: i32 = 10;
const DIAGONAL_WEIGHT: i32 = 14;

pub struct PathFindingPlugin;
impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PathFindingComponentsPlugin);

        app.add_observer(pathfinding_request_handler)
            .add_observer(check_visibility)
            .add_observer(handle_visibility_result)
            .add_observer(handle_pathfinding_results);

        app.add_observer(drop_debug_item);

        app.add_systems(
            Update,
            (pathfinding_cooldown_timer_handler, check_visibility_event),
        );
    }
}

/// Centralized handler for all InActionPathfindingTimer components
/// Used by follow, attack, dialog, and skill (in lua) pathfinding systems
fn pathfinding_cooldown_timer_handler(
    par_commands: ParallelCommands,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<InActionPathfindingTimer>)>,
) {
    let delta = time.delta();
    query.par_iter_mut().for_each(|(entity, mut timer)| {
        timer.tick(delta);

        if timer.just_finished() {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).remove::<InActionPathfindingTimer>();
            });
        }
    });
}

fn drop_debug_item(trigger: Trigger<DropDebugItem>, mut commands: Commands) -> Result<()> {
    let entity = trigger.target();
    let loc = trigger.event().0;

    let drop_item_packet = DropItem::new(entity.index(), 1.into(), 57.into(), loc, true, 1);

    commands.trigger_targets(GameServerPacket::from(drop_item_packet), entity);

    Ok(())
}

fn handle_pathfinding_results(path_result: Trigger<PathFindingResult>, mut commands: Commands) {
    let result = path_result.event();
    let requester_entity = path_result.target();

    let move_target = if result.path.is_empty() {
        log::debug!(
            "Path is empty, using goal as move target for <{:?}>: {:?}",
            requester_entity,
            result.goal
        );

        MoveTarget::single(WayPoint::new(result.start, result.goal))
    } else {
        MoveTarget::from(result.path.clone())
    };

    commands.entity(requester_entity).insert(move_target);
}

fn handle_visibility_result(
    visibility_result: Trigger<VisibilityCheckResult>,
    mut commands: Commands,
    world_map_query: WorldMapQuery,
) {
    let result = visibility_result.event();
    let requester_entity = visibility_result.target();
    if !result.is_visible {
        commands.trigger_targets(
            PathFindingRequest {
                start: result.start,
                goal: result.target,
                max_iterations: 500,
            },
            requester_entity,
        );
    } else {
        let mut can_move = result.is_visible;
        let geodata = world_map_query.region_geodata_from_pos(result.start);
        if let Ok(geodata) = geodata {
            can_move = geodata.can_move_to(
                &WorldMap::vec3_to_geo(result.start),
                &WorldMap::vec3_to_geo(result.target),
            );
        }
        if !can_move {
            commands.trigger_targets(
                PathFindingRequest {
                    start: result.start,
                    goal: result.target,
                    max_iterations: 500,
                },
                requester_entity,
            );
            return;
        }
        commands
            .entity(requester_entity)
            .try_insert(MoveTarget::single(WayPoint::new(
                result.start,
                result.target,
            )));
    }
}

fn check_visibility(
    visibility_request: Trigger<VisibilityCheckRequest>,
    world_map_query: WorldMapQuery,
    mut commands: Commands,
) -> Result<()> {
    let request = visibility_request.event();

    let start_region = RegionId::from(request.start);
    let goal_region = RegionId::from(request.target);

    let is_visible = if start_region != goal_region {
        // TODO: fix it later, now check real visibility only inside region.
        true
    } else {
        let geodata = world_map_query.region_geodata(start_region)?;
        geodata.can_move_to(
            &WorldMap::vec3_to_geo(request.start),
            &WorldMap::vec3_to_geo(request.target),
        )
    };

    commands.trigger_targets(
        VisibilityCheckResult {
            start: request.start,
            target: request.target,
            is_visible,
        },
        visibility_request.target(),
    );
    Ok(())
}

fn check_visibility_event(
    mut visibility_request: EventReader<VisibilityCheckRequest>,
    world_map_query: WorldMapQuery,
    mut commands: Commands,
) -> Result<()> {
    for request in visibility_request.read() {
        let start_region = RegionId::from(request.start);
        let goal_region = RegionId::from(request.target);

        let is_visible = if start_region != goal_region {
            // TODO: fix it later, now check real visibility only inside region.
            true
        } else {
            let geodata = world_map_query.region_geodata(start_region)?;
            geodata.can_move_to(
                &WorldMap::vec3_to_geo(request.start),
                &WorldMap::vec3_to_geo(request.target),
            )
        };

        commands.trigger_targets(
            VisibilityCheckResult {
                start: request.start,
                target: request.target,
                is_visible,
            },
            request.entity,
        );
    }
    Ok(())
}

pub fn pathfinding_request_handler(
    path_request: Trigger<PathFindingRequest>,
    world_map_query: WorldMapQuery,
    movables: Query<Ref<Movable>>,
    mut commands: Commands,
) {
    let request = path_request.event();
    let requester_entity = path_request.target();
    let start_region = RegionId::from(request.start);
    let goal_region = RegionId::from(request.goal);

    let in_water = movables
        .get(requester_entity)
        .map(|movable| movable.in_water())
        .unwrap_or(false);

    let is_flying = movables
        .get(requester_entity)
        .map(|movable| movable.is_flying())
        .unwrap_or(false);

    // Don't support pathfinding between different regions yet
    let different_regions = start_region != goal_region;

    let path = if different_regions || in_water || is_flying {
        let mut waypoints = SmallVec::new();
        waypoints.push(WayPoint::new(request.start, request.goal));
        waypoints
    } else {
        world_map_query
            .region_geodata(start_region)
            .map(|geodata| {
                find_path(
                    geodata,
                    WorldMap::vec3_to_geo(request.start),
                    WorldMap::vec3_to_geo(request.goal),
                    request.max_iterations,
                )
            })
            .unwrap_or_else(|_| {
                let mut waypoints = SmallVec::new();
                waypoints.push(WayPoint::new(request.start, request.goal));
                waypoints
            })
    };

    commands.trigger_targets(
        PathFindingResult {
            path,
            start: request.start,
            goal: request.goal,
        },
        requester_entity,
    );
}

fn find_path(
    geodata: &RegionGeoData,
    start: GeoVec3,
    goal: GeoVec3,
    max_iterations: usize,
) -> SmallVec<[WayPoint; WAYPOINTS_CAPACITY]> {
    if geodata.passable_directions(&start).is_empty() {
        log::trace!("Start position is not passable: {:?}", start);
        return SmallVec::new();
    }

    if geodata.passable_directions(&goal).is_empty() {
        log::trace!("Goal position is not passable: {:?}", goal);
        return SmallVec::new();
    }

    let start_to_goal_distance = start.manhattan_distance(&goal);
    log::trace!("Distance from start to goal: {}", start_to_goal_distance);

    let goal_point = goal.point();
    let mut iterations = 0;

    let result = pathfinding::prelude::astar(
        &start,
        |position| {
            iterations += 1;
            if iterations > max_iterations {
                SmallVec::new()
            } else {
                get_successors(geodata, position)
            }
        },
        |position| position.point().manhattan_distance(&goal_point),
        |position| position.point() == goal_point,
    );

    result
        .map(|(path, cost)| {
            log::trace!("Path found with {} points and cost {}", path.len(), cost);
            create_waypoints(geodata, path)
        })
        .unwrap_or_else(|| {
            log::trace!("No path found between {:?} and {:?}", start, goal);
            let mut waypoints = SmallVec::new();
            waypoints.push(WayPoint::new(
                WorldMap::geo_to_vec3(start),
                WorldMap::geo_to_vec3(goal),
            )); // Fallback to direct
            waypoints
        })
}

fn get_successors(geodata: &RegionGeoData, loc: &GeoVec3) -> SmallVec<[(GeoVec3, i32); 8]> {
    let mut successors = SmallVec::new();

    for direction in geodata.passable_directions(loc) {
        let neighbor_loc = loc.adjacent_position_in(direction, None);

        // Only consider fully open neighbors (all 8 directions passable)
        if geodata.passable_directions(&neighbor_loc).len() < 8 {
            continue;
        }

        // Check if we can actually step to this neighbor position
        // This validates height differences
        if !geodata.can_step_to(loc, &neighbor_loc) {
            continue;
        }

        let cost = if direction.is_diagonal() {
            DIAGONAL_WEIGHT
        } else {
            STRAIGHT_WEIGHT
        };
        successors.push((neighbor_loc, cost));
    }
    successors
}

fn create_waypoints(
    geodata: &RegionGeoData,
    path: Vec<GeoVec3>,
) -> SmallVec<[WayPoint; WAYPOINTS_CAPACITY]> {
    let mut simplified = SmallVec::<[GeoVec3; 128]>::new();
    let mut current_index = 0;

    simplified.push(path[current_index]);

    while current_index < path.len() - 1 {
        let mut farthest_reachable = current_index + 1;

        for check_index in (current_index + 1..path.len()).rev() {
            if geodata.can_move_to(&path[current_index], &path[check_index]) {
                farthest_reachable = check_index;
                break;
            }
        }

        simplified.push(path[farthest_reachable]);
        current_index = farthest_reachable;
    }

    simplified
        .windows(2)
        .map(|pair| {
            WayPoint::new(
                WorldMap::geo_to_vec3(pair[0]),
                WorldMap::geo_to_vec3(pair[1]),
            )
        })
        .collect::<SmallVec<[WayPoint; WAYPOINTS_CAPACITY]>>()
}
