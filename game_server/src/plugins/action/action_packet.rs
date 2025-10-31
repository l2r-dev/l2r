use bevy::{log, prelude::*};
use bevy_ecs::system::SystemParam;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::{
        pickup::PickupRequest,
        target::{SelectedTarget, Targetable},
    },
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

#[derive(SystemParam)]
struct ActionHandleParams<'w, 's> {
    receive_params: PacketReceiveParams<'w, 's>,
    character_query: Query<
        'w,
        's,
        (
            Option<Ref<'static, SelectedTarget>>,
            Ref<'static, KnownEntities>,
            Has<ActiveAction>,
        ),
        With<Character>,
    >,
    target_query: Query<
        'w,
        's,
        (
            Option<Ref<'static, Attackable>>,
            Option<Ref<'static, Transform>>,
            Has<Item>,
            Has<Character>,
            Has<Targetable>,
        ),
    >,
    object_id_manager: Res<'w, ObjectIdManager>,
    commands: Commands<'w, 's>,
}

fn handle_action_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    mut params: ActionHandleParams,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::Action(ref packet) = event.packet else {
        return Ok(());
    };

    let entity = params.receive_params.character(&event.connection.id())?;

    let (selected_target, known_entities, in_active_action) =
        params.character_query.get_mut(entity)?;

    let Some(packet_target_entity) = params.object_id_manager.entity(packet.object_id) else {
        params
            .commands
            .trigger_targets(GameServerPacket::from(ActionFail), entity);

        return Ok(());
    };

    let (_, _, is_item, _, is_targetable) = params.target_query.get(packet_target_entity)?;

    if is_item {
        if in_active_action {
            params
                .commands
                .entity(entity)
                .insert(NextIntention::PickUp {
                    item: packet_target_entity,
                });
        } else {
            // Insert PickupRequest - the pickup_request_handler will handle pathfinding
            params
                .commands
                .entity(entity)
                .insert(PickupRequest(packet_target_entity));
        }
        return Ok(());
    }

    if !is_targetable {
        params
            .commands
            .trigger_targets(GameServerPacket::from(ActionFail), entity);
        return Ok(());
    }

    // Handle existing selected target
    if let Some(curr_selected) = selected_target
        && **curr_selected == packet_target_entity
    {
        return handle_selected_target_action(
            packet.shift_pressed,
            entity,
            **curr_selected,
            in_active_action,
            params.target_query.reborrow(),
            params.commands.reborrow(),
        );
    }

    match known_entities.find_known_or_self(packet_target_entity, entity) {
        Some(found_entity) => {
            params
                .commands
                .entity(entity)
                .insert(SelectedTarget(found_entity));
        }
        None => {
            params
                .commands
                .trigger_targets(TargetNotFound(packet.object_id.into()), entity);
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

fn handle_selected_target_action(
    shift_pressed: bool,
    entity: Entity,
    curr_selected: Entity,
    in_active_action: bool,
    target_query: Query<(
        Option<Ref<Attackable>>,
        Option<Ref<Transform>>,
        Has<Item>,
        Has<Character>,
        Has<Targetable>,
    )>,
    mut commands: Commands,
) -> Result<()> {
    // Handle shift-click for NPC info dialog
    if shift_pressed {
        commands.trigger_targets(SendNpcInfoDialog(curr_selected), entity);
        return Ok(());
    }

    let (attackable, _, _, is_character, _) = target_query.get(curr_selected)?;

    match (attackable.is_some(), is_character, in_active_action) {
        // Attackable target
        (true, _, true) => {
            commands.entity(entity).insert(NextIntention::Attack {
                target: curr_selected,
            });
        }
        (true, _, false) => {
            commands.entity(entity).insert(Attacking(curr_selected));
        }
        // Character target (non-attackable)
        (false, true, true) => {
            commands.entity(entity).insert(NextIntention::Follow {
                target: curr_selected,
            });
        }
        (false, true, false) => {
            commands.trigger_targets(FollowRequest::from(curr_selected), entity);
        }
        // NPC target (dialog)
        (false, false, true) => {
            commands
                .entity(entity)
                .insert(NextIntention::DialogRequest {
                    target: curr_selected,
                });
        }
        (false, false, false) => {
            commands
                .entity(entity)
                .insert(DialogRequest::from(curr_selected));
        }
    }
    Ok(())
}
