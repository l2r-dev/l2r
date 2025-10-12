use bevy::prelude::*;

mod bypass;
mod client;
mod custom;
mod double_slash;
mod single_slash;

pub struct ChatCommandsPlugin;
impl Plugin for ChatCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(single_slash::SingleSlashCommandPlugin)
            .add_plugins(double_slash::BuildCommandsPlugin)
            .add_plugins(bypass::BypassCommandPlugin)
            .add_plugins(custom::CustomCommandsPlugin)
            .add_plugins(client::ClientCommandsPlugin);
    }
}
