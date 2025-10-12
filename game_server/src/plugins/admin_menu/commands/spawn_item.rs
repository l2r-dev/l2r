use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    items,
    network::packets::client::{BypassCommand, BypassCommandExecuted},
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut item_spawn: EventWriter<items::SpawnNew>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (_, selected_target) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::SpawnItem(item_id, count)) = cmd {
        let item_id = *item_id;
        item_spawn.write(items::SpawnNew {
            item_ids: vec![item_id],
            count: *count,
            item_location: items::ItemLocation::Inventory,
            dropped_entity: None,
            owner: Some(selected_target),
            silent: false, // Show system messages for admin-spawned items
        });
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
