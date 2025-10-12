use bevy::{ecs::system::ParallelCommands, prelude::*};
use game_core::{
    action::pickup::{PickupAnimation, PickupComponentsPlugin, PickupMetric, PickupRequest},
    animation::Animation,
    items::{AddInInventory, Item},
    movement::{ArrivedAtWaypoint, MoveTarget, MoveToEntity},
    network::{
        broadcast::{ServerPacketBroadcast, ServerPacketsBroadcast},
        packets::server::{DeleteObject, GetItem, StopMove},
    },
    object_id::ObjectId,
    path_finding::{InActionPathfindingTimer, VisibilityCheckRequest},
};
use l2r_core::metrics::Metrics;
use map::{WorldMap, WorldMapQuery};
use smallvec::smallvec;
use spatial::FlatDistance;
use state::GameServerStateSystems;

const PICKUP_DISTANCE: f32 = 5.0;

pub struct PickupPlugin;
impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        use l2r_core::metrics::MetricsAppExt;

        app.add_plugins(PickupComponentsPlugin);

        app.register_counter(PickupMetric::ItemsPickedUp, "Total items picked up");

        app.add_systems(
            Update,
            pickup_request_handler.in_set(GameServerStateSystems::Run),
        );
        app.add_systems(Update, pickup_animation_handler);
        app.add_observer(on_pickup_waypoint_arrival);
    }
}

fn pickup_request_handler(
    mut commands: Commands,
    characters: Query<
        (
            Entity,
            Ref<ObjectId>,
            Ref<Transform>,
            Ref<PickupRequest>,
            Option<Ref<MoveToEntity>>,
        ),
        Without<InActionPathfindingTimer>,
    >,
    items: Query<(Ref<ObjectId>, Ref<Item>, Ref<Transform>)>,
    world_map_query: WorldMapQuery,
) -> Result<()> {
    for (entity, char_oid, transform, request, move_to) in &mut characters.iter() {
        let item_entity = request.0;

        let Ok((item_oid, item, item_transform)) = items.get(item_entity) else {
            // Item no longer exists in world (picked up, destroyed, etc.)
            commands
                .entity(entity)
                .remove::<(PickupRequest, MoveToEntity)>();
            continue;
        };

        let char_pos = transform.translation;
        let item_pos = item_transform.translation;
        let distance = char_pos.flat_distance(&item_pos);

        // Item is within pickup range
        if distance <= PICKUP_DISTANCE {
            let geodata = world_map_query.region_geodata_from_pos(char_pos)?;

            // Check line of sight - can we see the item?
            if !geodata.can_see_target(
                WorldMap::vec3_to_geo(char_pos),
                WorldMap::vec3_to_geo(item_pos),
            ) {
                // Can't see item - remove pickup request and continue
                commands
                    .entity(entity)
                    .remove::<(MoveToEntity, PickupRequest)>();
                continue;
            }

            // Within range and line of sight is clear - start pickup animation
            commands
                .entity(entity)
                .remove::<(
                    PickupRequest,
                    MoveTarget,
                    MoveToEntity,
                    InActionPathfindingTimer,
                )>()
                .insert(PickupAnimation::new(item_entity));
            let get_item = GetItem::new(*char_oid, *item_oid, item.id(), item_pos).into();
            let stop_move = StopMove::new(*char_oid, *transform).into();
            commands.trigger_targets(
                ServerPacketsBroadcast::new(vec![stop_move, get_item].into()),
                entity,
            );
        } else {
            // Item is out of range, need to move closer
            // Check if already moving to the correct target
            if let Some(move_to) = move_to
                && move_to.target == item_entity
            {
                continue;
            }

            let geodata = world_map_query.region_geodata_from_pos(char_pos)?;

            // Use the same logic as follow/attack plugins - check line of sight
            let can_move_to = geodata.can_move_to(
                &WorldMap::vec3_to_geo(char_pos),
                &WorldMap::vec3_to_geo(item_pos),
            );

            if can_move_to {
                // Direct line of sight, use simple movement to item location
                commands
                    .entity(entity)
                    .insert(MoveTarget::single(spatial::WayPoint::new(
                        char_pos, item_pos,
                    )));
            } else {
                // No line of sight, use pathfinding via visibility check
                commands
                    .entity(entity)
                    .try_insert(InActionPathfindingTimer::default());

                commands.trigger_targets(
                    VisibilityCheckRequest {
                        entity,
                        start: char_pos,
                        target: item_pos,
                    },
                    entity,
                );
            }
        }
    }
    Ok(())
}

fn pickup_animation_handler(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<PickupAnimation>)>,
    object_ids: Query<Ref<ObjectId>, With<Item>>,
    metrics: Res<Metrics>,
    par_commands: ParallelCommands,
) -> Result<()> {
    query
        .par_iter_mut()
        .for_each(|(entity, mut pickup_animation)| {
            pickup_animation.timer_mut().tick(time.delta());
            if pickup_animation.timer().finished() {
                par_commands.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .remove::<(PickupAnimation, Animation)>();

                    if let Ok(counter) = metrics.counter(PickupMetric::ItemsPickedUp) {
                        counter.inc();
                    }

                    if let Ok(item_object_id) = object_ids.get(pickup_animation.entity()) {
                        commands.trigger_targets(
                            ServerPacketBroadcast::new(DeleteObject::new(*item_object_id).into()),
                            entity,
                        );
                        commands.trigger_targets(
                            AddInInventory::new(smallvec![pickup_animation.entity()]),
                            entity,
                        );
                    }
                });
            }
        });
    Ok(())
}

/// When a character with PickupRequest arrives at a waypoint (after pathfinding),
/// remove the InActionPathfindingTimer so the pickup_request_handler can process the pickup
fn on_pickup_waypoint_arrival(
    _arrived: Trigger<ArrivedAtWaypoint>,
    mut commands: Commands,
    query: Query<(Entity, &PickupRequest), With<InActionPathfindingTimer>>,
) {
    for (entity, _) in &query {
        commands.entity(entity).remove::<InActionPathfindingTimer>();
    }
}
