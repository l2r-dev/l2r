use bevy::prelude::*;

mod auth_login;
mod character_select;
mod enter_world;
mod net_ping;
mod protocol_verision;
mod request_logout;

pub(crate) struct AuthPlugin;
impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            net_ping::NetPingPlugin,
            protocol_verision::ClientProtocolVersionPlugin,
            auth_login::AuthLoginRequestPlugin,
            enter_world::EnterWorldPlugin,
            character_select::CharacterSelectPlugin,
            request_logout::RequestLogoutPlugin,
        ));
    }
}
