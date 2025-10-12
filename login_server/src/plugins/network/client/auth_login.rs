use super::LoginClientPacket;
use crate::{
    crypt::{LoginCryptEngine, LoginCryptParts},
    plugins::{
        accounts::model,
        network::{
            LoginServerNetworkConfig, LoginServerSession,
            server::{
                LoginServerPacket,
                login_fail::{LoginFail, LoginFailReason},
                login_ok::LoginOk,
            },
        },
    },
};
use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use bevy_slinet::server::PacketReceiveEvent;
use l2r_core::{
    crypt::session_keys::{SessionAccount, SessionKey},
    db::{RedisClient, Repository, RepositoryManager, TypedRepositoryManager},
    model::session::{L2rSession, ServerSessions},
    packets::{ClientPacketBuffer, L2rSerializeError},
    utils::log_trace_byte_table,
};
use rand::rngs::StdRng;
use rsa::{BigUint, hazmat::rsa_decrypt};
use sea_orm::{ActiveValue::Set, IntoActiveModel, TryIntoModel, entity::prelude::*};
use std::{convert::TryFrom, fmt::Debug, net::SocketAddr};

#[derive(Clone, PartialEq, Reflect)]
pub struct AuthLoginRequest {
    pub login: Option<String>,
    pub password: Option<String>,
    bytes: Vec<u8>,
}

impl Debug for AuthLoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthLoginRequest")
            .field("login", &self.login.as_deref().unwrap_or("<not decoded>"))
            .field(
                "password",
                &self.password.as_deref().unwrap_or("<not decoded>"),
            )
            .field("bytes_len", &self.bytes.len())
            .finish()
    }
}

impl AuthLoginRequest {
    pub fn decode(mut self, crypt_parts: &LoginCryptParts) -> Result<Self, L2rSerializeError> {
        log_trace_byte_table(&self.bytes, "AuthLoginRequest from_bytes");
        let encrypted = BigUint::from_bytes_be(&self.bytes[1..129]);
        log_trace_byte_table(&self.bytes[1..129], "before RSA decrypt");
        let rsa_key = &crypt_parts.rsa_key.private_key;
        let decrypted = rsa_decrypt::<StdRng>(None, rsa_key, &encrypted)
            .map_err(|e| {
                L2rSerializeError::with_source(
                    "RSA decrypt failed".to_string(),
                    e,
                    self.bytes.clone(),
                )
            })?
            .to_bytes_be();
        log_trace_byte_table(&decrypted, "after RSA decrypt");

        let login = LoginCryptEngine::read_string_from_bytes(&decrypted[3..])?;
        let password = LoginCryptEngine::read_string_from_bytes(&decrypted[17..])?;

        self.login = Some(login);
        self.password = Some(password);
        Ok(self)
    }
}

impl TryFrom<ClientPacketBuffer> for AuthLoginRequest {
    type Error = L2rSerializeError;

    fn try_from(buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        Ok(AuthLoginRequest {
            login: None,
            password: None,
            bytes: buffer.into(),
        })
    }
}

pub(crate) struct AuthLoginRequestPlugin;

impl Plugin for AuthLoginRequestPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<LoginServerNetworkConfig>>,
    login_sessions: Res<ServerSessions>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    if let LoginClientPacket::AuthLogin(ref packet) = event.packet {
        {
            let packet = packet.clone();
            let last_ip = event.connection.peer_addr();
            let session_entity = login_sessions.by_connection(&event.connection.id())?;
            commands.spawn_task(async move || handle_task(packet, session_entity, last_ip).await);
        }
    }
    Ok(())
}

async fn handle_task(
    packet: AuthLoginRequest,
    entity: Entity,
    last_ip: SocketAddr,
) -> Result<(), AccessError> {
    let accounts_repository = match AsyncWorld
        .resource::<RepositoryManager>()
        .get(|repo_manager| repo_manager.typed::<Uuid, model::Entity>())?
    {
        Ok(repo) => repo,
        Err(err) => {
            log::error!("{:?}", err);
            return Ok(());
        }
    };

    let Some(login) = packet.login.clone() else {
        log::error!("Login is missing");
        return Ok(());
    };

    let existing_account = accounts_repository
        .find_with_conditions([model::Column::Name.eq(login.clone())])
        .await
        .map(|accounts| accounts.first().cloned());

    let session = AsyncWorld
        .entity(entity)
        .component::<LoginServerSession>()
        .get(|session| session.clone())?;

    let Some(plaintext_password) = packet.password.as_deref() else {
        send_login_fail_and_disconnect(entity, LoginFailReason::SystemErrorLoginLater).await?;
        return Ok(());
    };

    let account_model = match existing_account {
        Ok(acc) => match acc {
            Some(acc) => acc,
            None => {
                let new_account = match model::Model::new(login, plaintext_password) {
                    Ok(account) => account,
                    Err(err) => {
                        error!("Failed to create new account model: {}", err);
                        send_login_fail_and_disconnect(
                            entity,
                            LoginFailReason::SystemErrorLoginLater,
                        )
                        .await?;
                        session.disconnect();
                        return Ok(());
                    }
                };

                let create_result = accounts_repository.create(&new_account).await;
                match create_result {
                    Ok(inserted_account) => inserted_account,
                    Err(err) => {
                        error!("Database error when creating account: {}", err);
                        send_login_fail_and_disconnect(
                            entity,
                            LoginFailReason::SystemErrorLoginLater,
                        )
                        .await?;
                        session.disconnect();
                        return Ok(());
                    }
                }
            }
        },
        Err(err) => {
            error!("Database error when fetching account: {:?}", err);
            send_login_fail_and_disconnect(entity, LoginFailReason::SystemErrorLoginLater).await?;
            session.disconnect();
            return Ok(());
        }
    };

    // write last_ip into accounts_repository
    // let last_ip = event.connection().remote_addr();
    // accounts_repository.update(&account_model.id(), |account| {
    //     account.last_ip = Some(last_ip);
    // }).await?;

    match account_model.verify_password(plaintext_password) {
        Ok(true) => {
            // Password is correct, continue with login
        }
        Ok(false) => {
            warn!(
                "Password is incorrect for account: {}",
                account_model.name()
            );
            send_login_fail_and_disconnect(
                entity,
                LoginFailReason::PasswordDoesNotMatchThisAccount,
            )
            .await?;
            session.disconnect();
            return Ok(());
        }
        Err(err) => {
            error!("Password verification error: {}", err);
            send_login_fail_and_disconnect(entity, LoginFailReason::SystemErrorLoginLater).await?;
            session.disconnect();
            return Ok(());
        }
    }

    let session_key = SessionKey::new();

    let account_session = SessionAccount {
        id: account_model.id(),
        access: account_model.access_level(),
        key: session_key,
    };

    let account_key = format!("account:{}:session", account_model.name());

    let session_json = match serde_json::to_string(&account_session) {
        Ok(json) => json,
        Err(err) => {
            log::error!("Failed to serialize session: {:?}", err);
            send_login_fail_and_disconnect(entity, LoginFailReason::SystemErrorLoginLater).await?;
            session.disconnect();
            return Ok(());
        }
    };

    let redis_result = AsyncWorld
        .resource::<RedisClient>()
        .get_mut(|redis_client| {
            redis::pipe()
                .atomic()
                .set(&account_key, &session_json)
                .ignore()
                .expire(&account_key, 300) // TTL - 5 minutes
                .query::<()>(&mut redis_client.connection)
        })?;

    if redis_result.is_err() {
        log::error!("Redis error: {:?}", redis_result);
        send_login_fail_and_disconnect(entity, LoginFailReason::SystemErrorLoginLater).await?;
        session.disconnect();
        return Ok(());
    }

    let mut active_account_model = account_model.into_active_model();
    active_account_model.last_ip = Set(Some(last_ip.to_string()));
    accounts_repository.update(&active_account_model).await?;

    let account_model = active_account_model
        .try_into_model()
        .map_err(|_| AccessError::ShouldNotHappen)?;

    AsyncWorld.entity(entity).insert(account_model)?;

    AsyncWorld.entity(entity).insert(session_key)?;

    AsyncWorld.apply_command(move |world: &mut World| {
        world.trigger_targets(LoginServerPacket::from(LoginOk::new(session_key)), entity)
    });

    Ok(())
}

async fn send_login_fail_and_disconnect(
    entity: Entity,
    reason: LoginFailReason,
) -> Result<(), AccessError> {
    let session = AsyncWorld
        .entity(entity)
        .component::<LoginServerSession>()
        .get(|session| session.clone())?;

    let lf = LoginFail::new(session.id(), reason);

    AsyncWorld.apply_command(move |world: &mut World| {
        world.trigger_targets(LoginServerPacket::from(lf), entity)
    });

    session.disconnect();

    AsyncWorld.entity(entity).despawn();
    Ok(())
}
