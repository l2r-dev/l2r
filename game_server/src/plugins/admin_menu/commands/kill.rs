use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    chat,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{CreatureSay, GameServerPacket},
    },
    stats::VitalsStats,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut vitals_stats: Query<Mut<VitalsStats>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (object_id, selected_target) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Kill) = cmd {
        let Ok(mut target_vitals) = vitals_stats.get_mut(selected_target) else {
            commands.trigger_targets(
                GameServerPacket::from(CreatureSay::new(
                    object_id,
                    "System".to_string(),
                    vec!["Target does not have VitalsStats and cant be dead.".to_string()],
                    chat::Kind::General,
                    None,
                )),
                entity,
            );
            commands.trigger_targets(LastAdminMenuPage, entity);
            return Ok(());
        };
        target_vitals.kill();
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
