use bevy::{ecs::query::QueryData, log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::pickup::PickupRequest,
    attack::Attacking,
    movement::{Following, MoveTarget},
    network::{
        broadcast::ServerPacketBroadcast,
        config::GameServerNetworkConfig,
        packets::{client::GameClientPacket, server::StopMove},
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
    path_finding::VisibilityCheckRequest,
    stats::Movable,
};

pub(crate) struct CannotMoveAnymorePlugin;

impl Plugin for CannotMoveAnymorePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovableObjectQuery<'a> {
    object_id: Ref<'a, ObjectId>,
    transform: Mut<'a, Transform>,
    move_target: Mut<'a, MoveTarget>,
    movable: Ref<'a, Movable>,
    following: Option<Ref<'a, Following>>,
    attacking: Option<Ref<'a, Attacking>>,
    pickup_request: Option<Ref<'a, PickupRequest>>,
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    mut movable_objects: Query<MovableObjectQuery, With<Movable>>,
    targets: Query<Ref<Transform>, Without<Movable>>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::CannotMoveAnymore(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;

        if let Ok(mut movable) = movable_objects.get_mut(character_entity) {
            movable.transform.translation = packet.location;
            movable.move_target.clear();

            log::debug!(
                "CannotMoveAnymore for entity {:?} at {:?}, heading: {}",
                character_entity,
                packet.location,
                packet.heading
            );

            // Broadcast stop move to nearby players
            let stop_packet = StopMove::new(*movable.object_id, *movable.transform);
            commands.trigger_targets(
                ServerPacketBroadcast::new(stop_packet.into()),
                character_entity,
            );

            // If following, attacking, or picking up, attempt pathfinding to reach target
            let target_entity = if let Some(following) = movable.following {
                Some(**following)
            } else if let Some(attacking) = movable.attacking {
                Some(**attacking)
            } else if let Some(pickup_request) = movable.pickup_request {
                Some(pickup_request.0)
            } else {
                None
            };

            if let Some(target_entity) = target_entity
                && let Ok(target_transform) = targets.get(target_entity)
            {
                let start_pos = packet.location;
                let target_pos = target_transform.translation;

                // Request visibility check which will trigger pathfinding if needed
                commands.trigger_targets(
                    VisibilityCheckRequest {
                        entity: character_entity,
                        start: start_pos,
                        target: target_pos,
                    },
                    character_entity,
                );
            }
        }
    }
    Ok(())
}
