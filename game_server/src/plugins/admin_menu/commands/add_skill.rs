use super::AdminCommandQuery;
use bevy::{log, prelude::*};
use game_core::{
    admin_menu::{AdminMenuCommand, LastAdminMenuPage},
    chat,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{CreatureSay, GameServerPacket},
    },
    skills::{Skill, SkillList},
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut skill_lists: Query<Mut<SkillList>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (object_id, target_entity) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::AddSkill(skill_id, skill_level)) = cmd {
        let skill_id = *skill_id;
        let skill_level = *skill_level;
        let Ok(mut skill_list) = skill_lists.get_mut(target_entity) else {
            log::error!("Failed to get skill list for entity {:?}", target_entity);
            commands.trigger_targets(
                GameServerPacket::from(CreatureSay::new(
                    object_id,
                    "System".to_string(),
                    vec!["Target does not have a skill list.".to_string()],
                    chat::Kind::General,
                    None,
                )),
                entity,
            );
            commands.trigger_targets(LastAdminMenuPage, entity);
            return Ok(());
        };
        let skill = Skill::new(skill_id, skill_level);
        skill_list.add_skill(skill);
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
