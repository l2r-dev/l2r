use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    character,
    custom_hierarchy::DespawnChildOf,
    items::InventoryLoad,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{CharacterSelected, GameServerPacket, SSQInfo},
        },
        session::GameServerSession,
    },
};
use l2r_core::model::session::{L2rSession, ServerSessions};

pub(crate) struct CharacterSelectPlugin;
impl Plugin for CharacterSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    sessions: Res<ServerSessions>,
    mut commands: Commands,
    mut query: Query<(Ref<GameServerSession>, Mut<character::Table>)>,
    mut inventory_load: EventWriter<InventoryLoad>,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::CharacterSelect(ref packet) = event.packet else {
        return Ok(());
    };
    let session_entity = sessions.by_connection(&event.connection.id())?;
    let (session, mut char_table) = query.get_mut(session_entity)?;

    // Despawn existing character if one is already active for this session
    if let Ok(existing_char_entity) = char_table.character() {
        log::warn!(
            "Character already spawned for session {:?}, despawning existing character before spawning new one",
            session.id()
        );
        commands.trigger_targets(character::CharacterSave, existing_char_entity);
        commands.entity(existing_char_entity).try_despawn();
        char_table.unset_character();
    }

    char_table.select(packet.char_slot)?;
    let selected_char = char_table.get_bundle()?.clone();
    let char_selected = CharacterSelected::new(&selected_char, session.id());
    let char_entity = commands.spawn(selected_char).id();
    char_table.set_character(char_entity);
    commands
        .entity(char_entity)
        .insert(DespawnChildOf(session_entity));
    commands.trigger_targets(GameServerPacket::from(SSQInfo::default()), session_entity);
    commands.trigger_targets(GameServerPacket::from(char_selected), session_entity);
    inventory_load.write(InventoryLoad::from(char_entity));
    Ok(())
}
