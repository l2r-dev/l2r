use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    character::Character,
    chat,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{CreatureSay, GameServerPacket},
    },
    stats::ProgressStats,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut progress_stats: Query<Mut<ProgressStats>, With<Character>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();

    let (object_id, target_entity) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::SetLevel(level)) = cmd {
        let level = *level;
        let mut stats = progress_stats.get_mut(target_entity)?;
        let mut message = format!("Level set to {}", level);
        if let Some(required_exp) = ProgressStats::get_exp_by_level(level) {
            stats.set_exp(required_exp);
        } else {
            message = format!("Level not found {}", level);
        }
        commands.trigger_targets(
            GameServerPacket::from(CreatureSay::new(
                object_id,
                "System".to_string(),
                vec![message],
                chat::Kind::General,
                None,
            )),
            entity,
        );
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
