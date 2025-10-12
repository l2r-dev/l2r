use bevy::prelude::*;
use game_core::{
    admin_menu::AdminMenuCommand,
    network::packets::client::{
        BypassCommand, BypassCommandExecuted, DoubleSlashCommand, DoubleSlashCommandExecuted,
    },
};

pub(super) fn handle(build_command: Trigger<DoubleSlashCommandExecuted>, mut commands: Commands) {
    let DoubleSlashCommandExecuted(cmd) = build_command.event();

    if cmd == &DoubleSlashCommand::Admin {
        let entity = build_command.target();

        commands.trigger_targets(
            BypassCommandExecuted(BypassCommand::Admin(AdminMenuCommand::Main)),
            entity,
        );
    }
}
