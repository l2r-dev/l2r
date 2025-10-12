use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    character,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, Restart, SendCharSelectionInfo},
        },
    },
};
use l2r_core::model::session::ServerSessions;

pub(crate) struct RequestRestartPlugin;
impl Plugin for RequestRestartPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    sessions: Res<ServerSessions>,
    mut character_tables: Query<Mut<character::Table>>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestRestart = event.packet else {
        return Ok(());
    };

    let session_entity = sessions.by_connection(&event.connection.id())?;
    let mut character_table = character_tables.get_mut(session_entity)?;
    let character_entity = character_table.character()?;

    commands.trigger_targets(character::CharacterSave, character_entity);

    commands.entity(character_entity).try_despawn();

    character_table.unset_character();

    // TODO: check if restart is allowed (eg. not in combat, not in a siege, etc.)
    let allowed = true;

    commands.trigger_targets(
        GameServerPacket::from(Restart::new(allowed)),
        session_entity,
    );

    commands.trigger_targets(SendCharSelectionInfo, session_entity);

    Ok(())
}
