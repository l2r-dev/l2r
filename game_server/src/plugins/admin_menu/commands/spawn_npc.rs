use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    network::packets::client::{BypassCommand, BypassCommandExecuted},
    npc,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    transforms: Query<Ref<Transform>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::SpawnNpc(npc_id)) = cmd {
        let current_transform = transforms.get(entity).map_err(|_| {
            BevyError::from(format!("Failed to get transform for entity: {:?}", entity))
        })?;
        commands.trigger_targets(
            npc::Spawn {
                id: *npc_id,
                transform: *current_transform,
            },
            entity,
        );
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
