mod chat_logger;
mod commands;
mod flood_protection;
mod say;

use bevy::prelude::*;
use game_core::chat::ChatComponentsPlugin;

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChatComponentsPlugin)
            .add_plugins(say::SayPlugin)
            .add_plugins(commands::ChatCommandsPlugin)
            .add_plugins(chat_logger::ChatLoggerPlugin)
            .add_plugins(flood_protection::ChatFloodProtectionPlugin);
    }
}
