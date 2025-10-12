use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    attack::Dead,
    chat,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{CreatureSay, GameServerPacket},
    },
    stats::Resurrect,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    dead: Query<Has<Dead>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (object_id, selected_target) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Resurrect) = cmd {
        if dead.get(selected_target).is_ok() {
            commands.trigger_targets(Resurrect, selected_target);
            commands.trigger_targets(LastAdminMenuPage, entity);
        } else {
            commands.trigger_targets(
                GameServerPacket::from(CreatureSay::new(
                    object_id,
                    "System".to_string(),
                    vec!["Not dead".to_string()],
                    chat::Kind::General,
                    None,
                )),
                entity,
            );
        }
    }
    Ok(())
}
