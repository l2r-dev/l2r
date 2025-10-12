use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    network::packets::client::{BypassCommand, BypassCommandExecuted},
};
use state::TogglePause;

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Pause) = cmd {
        commands.trigger_targets(TogglePause, entity);
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
