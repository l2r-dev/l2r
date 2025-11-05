use bevy::{log, platform::collections::HashMap, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    account::Account,
    character, items,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{AuthLoginRequest, GameClientPacket},
            server::{CharSelectionInfo, GameServerPacket},
        },
        session::{GameServerSession, PacketReceiveParams},
    },
    object_id::ObjectId,
};
use l2r_core::{
    crypt::session_keys::SessionAccount,
    db::{RedisClient, Repository, RepositoryManager, TypedRepositoryManager},
    model::session::L2rSession,
};
use sea_orm::prelude::*;

pub(crate) struct AuthLoginRequestPlugin;
impl Plugin for AuthLoginRequestPlugin {
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

    let GameClientPacket::AuthLoginRequest(packet) = &event.packet else {
        return Ok(());
    };

    let session_entity = receive_params.session(&event.connection.id())?;
    let packet = packet.clone();
    commands.spawn_task(async move || login_request_task(packet, session_entity).await);
    Ok(())
}

async fn login_request_task(packet: AuthLoginRequest, entity: Entity) -> Result<(), AccessError> {
    let session = AsyncWorld
        .entity(entity)
        .component::<GameServerSession>()
        .get(|session| session.clone())?;

    let session_key = format!("account:{}:session", packet.account);

    // Get session account from Redis
    let session_account = AsyncWorld
        .resource::<RedisClient>()
        .get_mut(|redis_client| {
            let redis_result: Result<String, _> =
                redis::Commands::get(&mut redis_client.connection, &session_key);

            match redis_result {
                Ok(json_str) => match serde_json::from_str::<SessionAccount>(&json_str) {
                    Ok(session_account) => Ok(session_account),
                    Err(err) => {
                        log::error!("#{:?} Failed to deserialize session: {:#?}", session, err);
                        session.disconnect();
                        Err(AccessError::Custom("Failed to deserialize session"))
                    }
                },
                Err(err) => {
                    log::error!("#{:?} Redis error: {:#?}", session, err);
                    session.disconnect();
                    Err(AccessError::Custom("Redis error"))
                }
            }
        })??;

    let account_id = session_account.id;
    let account_name = packet.account.clone();

    // Check if account is already logged in and disconnect existing session
    let existing_sessions: Vec<(Entity, GameServerSession, Option<Entity>)> =
        AsyncWorld.run(|world| {
            world
                .query::<(
                    Entity,
                    &Account,
                    &GameServerSession,
                    Option<&character::Table>,
                )>()
                .iter(world)
                .filter_map(|(session_entity, account, session, char_table_opt)| {
                    if account.id() == account_id {
                        let char_entity = char_table_opt.and_then(|t| t.character().ok());
                        Some((session_entity, session.clone(), char_entity))
                    } else {
                        None
                    }
                })
                .collect()
        });

    if !existing_sessions.is_empty() {
        log::warn!(
            "Account {} is already logged in. Disconnecting {} existing session(s).",
            account_name,
            existing_sessions.len()
        );

        for (_existing_entity, existing_session, _char_entity_opt) in existing_sessions {
            // Disconnect the old session (this will trigger despawn of session entity)
            existing_session.disconnect();
        }
    }

    // Create account and add to entity
    let account = Account::new(packet.account, session_account);
    AsyncWorld.entity(entity).insert(account.clone())?;

    let Ok(character_repository) = AsyncWorld
        .resource::<RepositoryManager>()
        .get(|manager| manager.typed::<ObjectId, character::model::Entity>())?
    else {
        return Ok(());
    };

    let Ok(items_repository) = AsyncWorld
        .resource::<RepositoryManager>()
        .get(|manager| manager.typed::<ObjectId, items::model::Entity>())?
    else {
        return Ok(());
    };

    let account_id = account.id();

    let chars_result = character_repository
        .find_with_conditions([character::model::Column::AccountId.eq(account_id)])
        .await;

    let chars = match chars_result {
        Ok(chars) => {
            let mut char_with_items: Vec<(
                character::model::Model,
                HashMap<ObjectId, items::model::Model>,
            )> = Vec::with_capacity(character::Table::MAX_CHARACTERS_ON_ACCOUNT);

            for character in chars {
                // Find all paperdoll items for this character
                let items_result = items_repository
                    .find_with_conditions([
                        items::model::Column::OwnerId.eq(character.id),
                        items::model::Column::Location.eq(items::ItemLocationVariant::PaperDoll),
                    ])
                    .await;

                match items_result {
                    Ok(items) => {
                        let items_map = items
                            .into_iter()
                            .map(|item| (item.object_id(), item))
                            .collect();
                        char_with_items.push((character, items_map));
                    }
                    Err(err) => {
                        log::error!(
                            "Error fetching items for character {}: {}",
                            character.id,
                            err
                        );
                        // Still include the character but with empty items
                        char_with_items.push((character, HashMap::new()));
                    }
                }
            }
            char_with_items
        }
        Err(err) => {
            log::error!("Database error fetching characters: {}", err);
            vec![]
        }
    };

    AsyncWorld.apply_command(move |world: &mut World| {
        let chars_table = character::Table::from_char_list(chars.clone(), session.id(), world);

        match chars_table {
            Ok(table) => {
                let char_selection_info = CharSelectionInfo::new(&table, chars);
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.insert(table);
                }
                world.trigger_targets(GameServerPacket::from(char_selection_info), entity);
            }
            Err(err) => {
                log::error!("Failed to create chars table: {}, disconnecting.", err);
                session.disconnect();
            }
        }
    });

    Ok(())
}
