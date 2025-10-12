use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::model::*,
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
};

pub(crate) struct RequestActionUsePlugin;
impl Plugin for RequestActionUsePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestActionUse(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;

        match packet.action_id {
            ActionId::Core(action) => {
                commands.trigger_targets(action, character_entity);
            }
            ActionId::Advanced(action) => {
                commands.trigger_targets(action, character_entity);
            }
            ActionId::Special(action) => {
                commands.trigger_targets(action, character_entity);
            }
        }
    }
    Ok(())
}
