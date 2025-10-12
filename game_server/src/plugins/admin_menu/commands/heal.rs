use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    network::packets::client::{BypassCommand, BypassCommandExecuted},
    stats::FullVitalsRestore,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (_, selected_target) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Heal) = cmd {
        commands.trigger_targets(FullVitalsRestore::from(selected_target), selected_target);
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
