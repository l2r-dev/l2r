use super::AdminCommandQuery;
use bevy::{log, prelude::*};
use game_core::{
    admin_menu::{AdminMenu, AdminMenuCommand, AdminMenuPage},
    items,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{ActionFail, GameServerPacket, NpcHtmlMessage, TeleportToLocation},
    },
    teleport::{TeleportDestinations, TeleportDestinationsHandle, TeleportType},
};
use l2r_core::{
    assets::html::{TeraContext, TeraHtmlTemplater},
    utils::PaginateSlice,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    teleport_dest_assets: Res<Assets<TeleportDestinations>>,
    teleport_dest_handle: Res<TeleportDestinationsHandle>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    let (object_id, _) = admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::Tp(tp_id)) = cmd {
        let Some(destinations_table) = teleport_dest_assets.get(teleport_dest_handle.id()) else {
            log::error!("Teleport destinations not found");
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return Ok(());
        };

        let Some(destination) = destinations_table.get(tp_id) else {
            log::error!("Teleport destination not found: {}", tp_id);
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return Ok(());
        };

        commands.trigger_targets(
            TeleportToLocation::new(
                object_id,
                Transform::from_translation(destination.location.into()),
                TeleportType::default(),
            ),
            entity,
        );
    }
    Ok(())
}

const DESTINATIONS_PER_PAGE: usize = 15;

pub(super) fn handle_list(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut admin_menu: ResMut<AdminMenu>,
    teleport_dest_handle: Res<TeleportDestinationsHandle>,
    teleport_dest_assets: Res<Assets<TeleportDestinations>>,
    characters: Query<Ref<Name>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();

    let (object_id, _) = admin_query.validate_gm(entity)?;

    if let BypassCommand::Admin(AdminMenuCommand::TpList(page)) = cmd {
        let name = characters.get(entity)?;
        let Some(destinations_table) = teleport_dest_assets.get(teleport_dest_handle.id()) else {
            log::error!("Teleport destinations not found {:?}", entity);
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return Ok(());
        };
        let page = *page;
        let all_destinations = destinations_table.all().collect::<Vec<_>>();
        let (destinations, pagination) = all_destinations
            .as_slice()
            .paginate(page as usize, DESTINATIONS_PER_PAGE);
        let mut context = tera::Context::new();
        context.insert("admin_name", &name.to_string());
        context.insert("admin_access_level", &admin_query.account(entity)?.access());
        context.insert("destinations", &destinations);
        context.extend(pagination.tera_context());

        let page_name = AdminMenuPage::TpList;
        let rendered = admin_menu.render_with_fallback(page_name.html().as_str(), &context)?;
        admin_menu.set_last_page(entity, page_name, Some(context.clone()));

        commands.trigger_targets(
            GameServerPacket::from(NpcHtmlMessage::new(
                object_id,
                rendered,
                items::Id::default(),
            )),
            entity,
        );
    }
    Ok(())
}
