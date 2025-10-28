use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::target::SelectedTarget,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{ActionFail, GameServerPacket},
        },
        session::PacketReceiveParams,
    },
};

pub(crate) struct RequestCancelTargetPlugin;

impl Plugin for RequestCancelTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    mut commands: Commands,
    receive_params: PacketReceiveParams,
    selected_target: Query<Has<SelectedTarget>>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestCancelTarget = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let has_selected_target = selected_target.get(character_entity)?;

        if has_selected_target {
            commands.entity(character_entity).remove::<SelectedTarget>();
            commands.trigger_targets(GameServerPacket::from(ActionFail), character_entity);
        } else {
            commands.trigger_targets(GameServerPacket::from(ActionFail), character_entity);
        }
    }
    Ok(())
}
