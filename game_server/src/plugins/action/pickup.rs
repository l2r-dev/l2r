use bevy::prelude::*;
use bevy_ecs::query::QueryData;
use game_core::{
    action::{
        pickup::{PickupComponentsPlugin, PickupMetric, PickupRequest},
        wait_kind::Sit,
    },
    active_action::ActiveAction,
    items::{AddInInventory, Item},
    movement::{ArrivedAtWaypoint, Movement},
    network::{
        broadcast::ServerPacketsBroadcast,
        packets::server::{ActionFail, GameServerPacket, GetItem},
    },
    object_id::ObjectId,
    path_finding::{DirectMoveRequest, InActionPathfindingTimer},
};
use l2r_core::metrics::Metrics;
use map::WorldMapQuery;
use spatial::FlatDistance;
use state::GameServerStateSystems;
use std::time::Duration;

const PICKUP_DISTANCE: f32 = 20.0;
const PICKUP_ACTION_DURATION: Duration = Duration::from_millis(500);

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
        app.add_observer(on_pickup_waypoint_arrival);
    }
}

#[derive(QueryData)]
struct CharacterQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
    request: Ref<'a, PickupRequest>,
    movement: Option<Ref<'a, Movement>>,
    is_sitting: Has<Sit>,
}

fn pickup_request_handler(
    mut commands: Commands,
    characters: Query<CharacterQuery, Without<InActionPathfindingTimer>>,
    items: Query<(Ref<ObjectId>, Ref<Item>, Ref<Transform>)>,
    map_query: WorldMapQuery,
    metrics: Res<Metrics>,
) -> Result<()> {
    for character in &mut characters.iter() {
        if character.is_sitting {
            commands
                .entity(character.entity)
                .remove::<(PickupRequest, Movement)>();

            commands.trigger_targets(GameServerPacket::from(ActionFail), character.entity);

            continue;
        }

        let item_entity = character.request.0;

        let Ok((item_oid, item, item_transform)) = items.get(item_entity) else {
            // Item no longer exists in world (picked up, destroyed, etc.)
            commands
                .entity(character.entity)
                .remove::<(PickupRequest, Movement)>();

            commands.trigger_targets(GameServerPacket::from(ActionFail), character.entity);

            continue;
        };

        let char_pos = character.transform.translation;
        let item_pos = item_transform.translation;
        let distance = char_pos.flat_distance(&item_pos);

        // Item is within pickup range
        if distance <= PICKUP_DISTANCE {
            // Check line of sight - can we see the item?
            if !map_query.can_see_target(char_pos, item_pos) {
                // Can't see item - remove pickup request and continue
                commands
                    .entity(character.entity)
                    .remove::<(Movement, PickupRequest)>();

                commands.trigger_targets(GameServerPacket::from(ActionFail), character.entity);

                continue;
            }

            if let Ok(counter) = metrics.counter(PickupMetric::ItemsPickedUp) {
                counter.inc();
            }

            // Within range and line of sight is clear - start pickup animation
            commands
                .entity(character.entity)
                .remove::<(PickupRequest, Movement, InActionPathfindingTimer)>()
                .insert(ActiveAction::new(PICKUP_ACTION_DURATION));

            commands.trigger_targets(AddInInventory::new(item_entity), character.entity);

            commands.trigger_targets(GameServerPacket::from(ActionFail), character.entity);

            let get_item =
                GetItem::new(*character.object_id, *item_oid, item.id(), item_pos).into();

            commands.trigger_targets(
                ServerPacketsBroadcast::new(vec![get_item].into()),
                character.entity,
            );
        } else {
            // Item is out of range, need to move closer
            // Check if already moving to the correct target
            if let Some(mov) = character.movement
                && mov.is_to_location()
            {
                // Already moving to a location (likely the item)
                continue;
            }

            if map_query.can_move_to(char_pos, item_pos) {
                // Direct line of sight, use simple movement to item location
                commands
                    .entity(character.entity)
                    .insert(Movement::to_waypoint(spatial::WayPoint::new(
                        char_pos, item_pos,
                    )));
            } else {
                // No line of sight, use pathfinding via visibility check
                commands
                    .entity(character.entity)
                    .try_insert(InActionPathfindingTimer::default());

                commands.trigger_targets(
                    DirectMoveRequest {
                        entity: character.entity,
                        start: char_pos,
                        target: item_pos,
                    },
                    character.entity,
                );
            }
        }
    }
    Ok(())
}

/// When a character with PickupRequest arrives at a waypoint (after pathfinding),
/// remove the InActionPathfindingTimer so the pickup_request_handler can process the pickup
fn on_pickup_waypoint_arrival(arrived: Trigger<ArrivedAtWaypoint>, mut commands: Commands) {
    let arrived_entity = arrived.target();
    commands
        .entity(arrived_entity)
        .try_remove::<InActionPathfindingTimer>();
}
