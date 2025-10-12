use bevy::prelude::*;

mod request_char_create;
mod request_char_delete;
mod request_goto_lobby;
mod request_menu;

pub struct CharacterCreationPlugin;
impl Plugin for CharacterCreationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            request_char_create::RequestCharCreatePlugin,
            request_char_delete::RequestCharDeletePlugin,
            request_menu::RequestCharCreateMenuPlugin,
            request_goto_lobby::RequestGotoLobbyPlugin,
        ));
    }
}
