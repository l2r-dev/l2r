use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::target::SelectedTarget,
    animation::Animation,
    attack::{Attacking, InsertAttackingParams},
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    object_id::ObjectIdManager,
    player_specific::next_intention::NextIntention,
};

pub struct AttackPacketPlugin;

impl Plugin for AttackPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_attack_packet);
    }
}

fn handle_attack_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    query: Query<(Ref<SelectedTarget>, Has<Animation>)>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    let event = receive.event();

    let GameClientPacket::Attack(ref packet) = event.packet else {
        return Ok(());
    };

    let attacker_entity = receive_params.character(&event.connection.id())?;

    let Ok((selected_target, has_animation)) = query.get(attacker_entity) else {
        return Ok(());
    };

    let Some(packet_entity) = object_id_manager.entity(packet.object_id) else {
        return Ok(());
    };

    let target_entity = **selected_target;

    if target_entity == packet_entity {
        if has_animation {
            commands
                .entity(attacker_entity)
                .insert(NextIntention::Attack {
                    target: target_entity,
                });
        } else {
            Attacking::insert(
                &mut commands,
                InsertAttackingParams {
                    attacker: attacker_entity,
                    target: target_entity,
                },
            )
        }
    }

    Ok(())
}
