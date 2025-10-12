use self::{
    auth_gg::AuthGGRequest, auth_login::AuthLoginRequest, server_list::ServerListRequest,
    server_login::GameServerLoginRequest,
};
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, ClientPacketId, L2rClientPacket, L2rSerializeError};
use std::convert::TryFrom;

mod auth_gg;
pub mod auth_login;
mod server_list;
mod server_login;

pub(crate) struct LoginClientPacketPlugin;
impl Plugin for LoginClientPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(auth_login::AuthLoginRequestPlugin)
            .add_plugins(auth_gg::AuthGGRequestPlugin)
            .add_plugins(server_login::GameServerLoginRequestPlugin)
            .add_plugins(server_list::ServerListRequestPlugin);
    }
}

pub struct LoginClientPacketCode;
impl LoginClientPacketCode {
    pub const AUTH_LOGIN_REQUEST: ClientPacketId = ClientPacketId::new(0x00);
    pub const GAME_SERVER_LOGIN_REQUEST: ClientPacketId = ClientPacketId::new(0x02);
    pub const SERVER_LIST_REQUEST: ClientPacketId = ClientPacketId::new(0x05);
    pub const AUTH_GG_REQUEST: ClientPacketId = ClientPacketId::new(0x07);
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum LoginClientPacket {
    AuthGG(AuthGGRequest),
    AuthLogin(AuthLoginRequest),
    ServerList(ServerListRequest),
    GameServerLogin(GameServerLoginRequest),
}
impl Default for LoginClientPacket {
    fn default() -> Self {
        LoginClientPacket::AuthGG(AuthGGRequest::default())
    }
}

impl TryFrom<ClientPacketBuffer> for LoginClientPacket {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        match ClientPacketId::from(&mut buffer) {
            LoginClientPacketCode::AUTH_GG_REQUEST => {
                Ok(Self::AuthGG(AuthGGRequest::try_from(buffer)?))
            }
            LoginClientPacketCode::AUTH_LOGIN_REQUEST => {
                Ok(Self::AuthLogin(AuthLoginRequest::try_from(buffer)?))
            }
            LoginClientPacketCode::SERVER_LIST_REQUEST => {
                Ok(Self::ServerList(ServerListRequest::try_from(buffer)?))
            }
            LoginClientPacketCode::GAME_SERVER_LOGIN_REQUEST => Ok(Self::GameServerLogin(
                GameServerLoginRequest::try_from(buffer)?,
            )),
            _ => Err(L2rSerializeError::new(
                "Invalid packet id".to_string(),
                buffer.as_slice(),
            )),
        }
    }
}

impl L2rClientPacket for LoginClientPacket {}
