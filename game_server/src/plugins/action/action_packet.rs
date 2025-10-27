use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::{pickup::PickupRequest, target::SelectedTarget},
    active_action::ActiveAction,
    attack::{Attackable, Attacking},
    character::Character,
    encounters::KnownEntities,
    items::Item,
    movement::FollowRequest,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{GameClientPacket, TargetNotFound},
            server::{ActionFail, GameServerPacket},
        },
        session::PacketReceiveParams,
    },
    npc::{DialogRequest, SendNpcInfoDialog},
    object_id::ObjectIdManager,
    player_specific::next_intention::NextIntention,
};

pub(crate) struct ActionPacketPlugin;
impl Plugin for ActionPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TargetNotFound>();

        app.add_observer(handle_action_packet)
            .add_observer(target_not_found);
    }
}

fn handle_action_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut character_query: Query<
        (
            Option<Ref<SelectedTarget>>,
            Ref<KnownEntities>,
            Has<ActiveAction>,
        ),
        With<Character>,
    >,
    target_query: Query<(
        Option<Ref<Attackable>>,
        Option<Ref<Transform>>,
        Has<Item>,
        Has<Character>,
    )>,
    object_id_manager: Res<ObjectIdManager>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::Action(ref packet) = event.packet else {
        return Ok(());
    };

    let entity = receive_params.character(&event.connection.id())?;

    let (selected_target, known_entities, in_active_action) = character_query.get_mut(entity)?;

    // Check if target is an item first
    if let Some(packet_target_entity) = object_id_manager.entity(packet.object_id) {
        let (_, _, is_item, _) = target_query.get(packet_target_entity)?;
        if is_item {
            if in_active_action {
                commands.entity(entity).insert(NextIntention::PickUp {
                    item: packet_target_entity,
                });
            } else {
                // Insert PickupRequest - the pickup_request_handler will handle pathfinding
                commands
                    .entity(entity)
                    .insert(PickupRequest(packet_target_entity));
            }

            return Ok(());
        }
    }

    let Some(packet_target_entity) = object_id_manager.entity(packet.object_id) else {
        commands.trigger_targets(GameServerPacket::from(ActionFail), entity);

        return Ok(());
    };

    // Handle existing selected target
    if let Some(curr_selected) = selected_target {
        let curr_selected = **curr_selected;

        if curr_selected == packet_target_entity {
            if packet.shift_pressed {
                commands.trigger_targets(SendNpcInfoDialog(curr_selected), entity);
                return Ok(());
            }

            let (attackable, _, _, is_character) = target_query.get(packet_target_entity)?;

            if attackable.is_some() {
                if in_active_action {
                    commands.entity(entity).insert(NextIntention::Attack {
                        target: curr_selected,
                    });
                } else {
                    commands.entity(entity).insert(Attacking(curr_selected));
                }
            } else if is_character {
                if in_active_action {
                    commands.entity(entity).insert(NextIntention::Follow {
                        target: curr_selected,
                    });
                } else {
                    commands.trigger_targets(FollowRequest::from(packet_target_entity), entity);
                }
            } else if in_active_action {
                commands
                    .entity(entity)
                    .insert(NextIntention::DialogRequest {
                        target: curr_selected,
                    });
            } else {
                commands
                    .entity(entity)
                    .insert(DialogRequest::from(packet_target_entity));
            }

            return Ok(());
        }
    }

    match known_entities.find_known_or_self(packet_target_entity, entity) {
        Some(found_entity) => {
            commands.entity(entity).insert(SelectedTarget(found_entity));
        }
        None => {
            commands.trigger_targets(TargetNotFound(packet.object_id.into()), entity);
        }
    }
    Ok(())
}

fn target_not_found(not_found: Trigger<TargetNotFound>) {
    let target_index = **not_found.event();
    let session_entity = not_found.target();

    log::error!(
        "No target found in range for session_entity={:?}, target_index={:?}",
        session_entity,
        target_index
    );
}
