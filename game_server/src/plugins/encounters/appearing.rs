use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    teleport::TeleportInProgress,
};

pub(crate) struct AppearingPlugin;
impl Plugin for AppearingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(appearing_handle);
    }
}

fn appearing_handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    mut movable_objects: Query<Entity, With<TeleportInProgress>>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::Appearing = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let entity = movable_objects.get_mut(character_entity)?;
        commands.entity(entity).remove::<TeleportInProgress>();
    }
    Ok(())
}
