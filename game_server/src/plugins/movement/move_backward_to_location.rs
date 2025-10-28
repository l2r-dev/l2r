use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::pickup::PickupRequest,
    active_action::ActiveAction,
    attack::Attacking,
    movement::Following,
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    npc::DialogRequest,
    path_finding::VisibilityCheckRequest,
    player_specific::next_intention::NextIntention,
    stats::Movable,
};

pub(crate) struct MoveBackwardToLocationPlugin;

impl Plugin for MoveBackwardToLocationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    movable_objects: Query<(&Transform, Has<ActiveAction>), With<Movable>>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::MoveBackwardToLocation(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;

        if let Ok((transform, has_active_active_action)) = movable_objects.get(character_entity) {
            // Cancel any active actions when player manually moves to a different location
            if has_active_active_action {
                commands
                    .entity(character_entity)
                    .insert(NextIntention::MoveTo {
                        start: transform.translation,
                        target: packet.target_location,
                    });
            } else {
                commands.entity(character_entity).remove::<(
                    PickupRequest,
                    Following,
                    Attacking,
                    DialogRequest,
                )>();
                commands.trigger_targets(
                    VisibilityCheckRequest {
                        entity: character_entity,
                        start: transform.translation,
                        target: packet.target_location,
                    },
                    character_entity,
                );
            }
        }
    }
    Ok(())
}
