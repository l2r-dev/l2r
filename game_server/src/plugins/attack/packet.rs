use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::target::SelectedTarget,
    attack::Attacking,
    movement::{Following, MoveTarget},
    network::{
        config::GameServerNetworkConfig,
        packets::client::{AttackRequest, GameClientPacket},
        session::PacketReceiveParams,
    },
    object_id::ObjectIdManager,
};

pub struct AttackPacketPlugin;

impl Plugin for AttackPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AttackRequest>();
        app.add_observer(handle_attack_packet);
        app.add_observer(handle_attack_request);
    }
}

fn handle_attack_request(attack_req: Trigger<AttackRequest>, mut commands: Commands) {
    let target = **attack_req.event();
    let attacker = attack_req.target();
    commands
        .entity(attacker)
        .remove::<MoveTarget>()
        .remove::<Following>()
        .insert(Attacking(target));
}

fn handle_attack_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    query: Query<Ref<SelectedTarget>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    object_id_manager: Res<ObjectIdManager>,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::Attack(ref packet) = event.packet else {
        return Ok(());
    };
    let entity = receive_params.character(&event.connection.id())?;

    let selected_target = match query.get(entity) {
        Ok(data) => data,
        Err(_) => return Ok(()),
    };

    let Some(packet_entity) = object_id_manager.entity(packet.object_id) else {
        return Ok(());
    };

    let selected_entity = **selected_target;

    if selected_entity == packet_entity {
        commands.trigger_targets(AttackRequest::from(selected_entity), entity);
    }
    Ok(())
}
