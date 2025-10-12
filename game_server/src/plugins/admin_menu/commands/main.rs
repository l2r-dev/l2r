use super::AdminCommandQuery;
use bevy::prelude::*;
use game_core::{
    admin_menu::{AdminMenu, AdminMenuCommand, AdminMenuPage},
    items,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{GameServerPacket, NpcHtmlMessage},
    },
};
use l2r_core::assets::html::TeraHtmlTemplater;

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut admin_menu: ResMut<AdminMenu>,
    characters: Query<Ref<Name>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (object_id, _) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Main) = cmd {
        let name = characters.get(entity)?;
        let mut context = tera::Context::new();
        context.insert("admin_name", &name.to_string());
        context.insert("admin_access_level", &admin_query.account(entity)?.access());
        let page = AdminMenuPage::MainMenu;
        let html = admin_menu.render_with_fallback(page.html().as_str(), &context)?;
        admin_menu.set_last_page(entity, page, Some(context.clone()));
        commands.trigger_targets(
            GameServerPacket::from(NpcHtmlMessage::new(object_id, html, items::Id::default())),
            entity,
        );
    }
    Ok(())
}
