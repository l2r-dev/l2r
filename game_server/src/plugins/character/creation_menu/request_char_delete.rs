use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use bevy_ecs::system::SystemState;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    account::Account,
    character,
    items::ItemsQuery,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{GameClientPacket, RequestCharDelete},
            server::*,
        },
    },
    object_id::{ObjectId, ObjectIdManager},
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    model::session::ServerSessions,
};

pub(crate) struct RequestCharDeletePlugin;
impl Plugin for RequestCharDeletePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    sessions: Res<ServerSessions>,
    mut commands: Commands,
    mut query: Query<(&Account, Mut<character::Table>)>,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestCharDelete(ref packet) = event.packet else {
        return Ok(());
    };
    let session_entity = sessions.by_connection(&event.connection.id())?;
    let (account, chars_table) = query.get_mut(session_entity)?;
    let char_slot = packet.char_slot();
    if !chars_table.is_valid_slot(char_slot) {
        log::warn!(
            "Invalid char slot {} for account {:?}, only have {} chars",
            char_slot,
            account.id(),
            chars_table.len()
        );
        commands.trigger_targets(
            GameServerPacket::from(CharacterDeletionFailed::new(
                CharacterDeletionFailReason::DeletionFailed,
            )),
            session_entity,
        );
        return Ok(());
    }
    let char_delete_request = *packet;
    commands
        .spawn_task(move || async move { delete_task(session_entity, char_delete_request).await });
    Ok(())
}

async fn delete_task(
    session_entity: Entity,
    char_delete_request: RequestCharDelete,
) -> Result<(), AccessError> {
    let Ok(character_repository) = AsyncWorld
        .resource::<RepositoryManager>()
        .get(|manager| manager.typed::<ObjectId, character::model::Entity>())?
    else {
        return Ok(());
    };

    let char_info = AsyncWorld
        .entity(session_entity)
        .component::<character::Table>()
        .get_mut(|table| {
            table.select(char_delete_request.char_slot()).ok()?;
            table
                .get_bundle()
                .ok()
                .map(|bundle| (bundle.id, bundle.name.to_string()))
        })?;
    if char_info.is_none() {
        log::error!(
            "No character found at slot {} for session entity {:?}",
            char_delete_request.char_slot(),
            session_entity
        );
        send_fail_reason_to_session(CharacterDeletionFailReason::default(), session_entity).await;
        return Ok(());
    }

    let (char_id, char_name) = char_info.unwrap();
    let exists = character_repository.find_by_id(char_id).await;
    if exists.is_err() {
        log::error!("Character {} ({}) not found in DB", char_name, char_id);
        send_fail_reason_to_session(CharacterDeletionFailReason::default(), session_entity).await;
        return Ok(());
    }

    let delete_res = character_repository.delete_by_id(char_id).await;
    if delete_res.is_err() {
        log::error!(
            "Failed to delete character {} ({}) from DB",
            char_name,
            char_id
        );
        send_fail_reason_to_session(CharacterDeletionFailReason::default(), session_entity).await;
        return Ok(());
    }

    log::info!("Character {} ({}) deleted from DB", char_name, char_id);
    delete_from_char_table(session_entity, char_delete_request).await?;
    AsyncWorld
        .resource::<ObjectIdManager>()
        .get_mut(|manager| manager.release_id(char_id))?;

    Ok(())
}

async fn delete_from_char_table(
    session_entity: Entity,
    request: RequestCharDelete,
) -> Result<(), AccessError> {
    let bundle_result = AsyncWorld
        .entity(session_entity)
        .component::<character::Table>()
        .get_mut(|table| table.remove_slot(request.char_slot()).ok())?;

    if bundle_result.is_none() {
        AsyncWorld.apply_command(move |world: &mut World| {
            world.trigger_targets(
                GameServerPacket::from(CharacterDeletionFailed::new(
                    CharacterDeletionFailReason::DeletionFailed,
                )),
                session_entity,
            );
        });
    } else {
        AsyncWorld.apply_command(move |world: &mut World| {
            world.trigger_targets(
                GameServerPacket::from(CharacterDeletionSuccess),
                session_entity,
            );
        });
    }

    AsyncWorld.apply_command(move |world: &mut World| {
        let mut items_state: SystemState<ItemsQuery> = SystemState::new(world);
        let items_query = items_state.get(world);
        let table = world.entity(session_entity).get::<character::Table>();

        if let Some(table) = table {
            world.trigger_targets(
                GameServerPacket::from(CharSelectionInfo::from_query(table, &items_query)),
                session_entity,
            );
        } else {
            log::error!(
                "Failed to get character table for entity {:?}",
                session_entity
            );
        }
    });

    Ok(())
}

async fn send_fail_reason_to_session(
    fail_reason: CharacterDeletionFailReason,
    session_entity: Entity,
) {
    AsyncWorld.apply_command(move |world: &mut World| {
        world.trigger_targets(
            GameServerPacket::from(CharacterDeletionFailed::new(fail_reason)),
            session_entity,
        );
    });
}
