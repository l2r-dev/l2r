use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    character,
    network::{
        config::GameServerNetworkConfig,
        packets::client::{DoubleSlashCommandExecuted, GameClientPacket},
        session::GetCharEntity,
    },
};
use l2r_core::model::session::ServerSessions;

mod admin;
mod go_to;
mod spawn;

pub struct BuildCommandsPlugin;
impl Plugin for BuildCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_packet);
        app.add_observer(admin::handle)
            .add_observer(spawn::handle)
            .add_observer(go_to::handle);
    }
}

fn handle_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    sessions: Res<ServerSessions>,
    character_tables: Query<Ref<character::Table>>,
    mut commands: Commands,
) {
    let event = receive.event();

    if let GameClientPacket::DoubleSlashCommand(ref packet) = event.packet
        && let Ok(character_entity) =
            sessions.get_character_entity(&event.connection.id(), &character_tables)
    {
        commands.trigger_targets(DoubleSlashCommandExecuted::from(*packet), character_entity);
    }
}
