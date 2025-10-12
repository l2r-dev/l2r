use super::AdminCommandQuery;
use bevy::{log, platform::collections::HashMap, prelude::*};
use game_core::{
    admin_menu::{AdminMenu, AdminMenuCommand, AdminMenuPage, LastAdminMenuPage},
    items::{Grade, Id as ItemId, Kind},
    multisell::admin_shop::{AdminShopMultiSells, CategoryMultisell, NamedMultisell},
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{GameServerPacket, MultisellList, NpcHtmlMessage},
    },
};
use l2r_core::{
    assets::html::{TeraContext, TeraHtmlTemplater},
    utils::PaginateSlice,
};

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    admin_shop: Res<AdminShopMultiSells>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    admin_query.validate_gm(entity)?;
    if let BypassCommand::Admin(AdminMenuCommand::MultiSell(multisell_id)) = cmd {
        log::debug!(
            "AdminMenu: Entity {:?} called multisell id {}",
            entity,
            multisell_id
        );
        let packets = MultisellList::multipage_list(
            *multisell_id,
            admin_shop.get(multisell_id).cloned().unwrap_or_default(),
        );
        commands.trigger_targets(packets, entity);
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}

fn create_multisell_categories(admin_shop: &AdminShopMultiSells) -> Vec<CategoryMultisell> {
    let mut category_map: HashMap<String, Vec<(Kind, Grade)>> =
        HashMap::with_capacity(admin_shop.len());
    for multisell_id in admin_shop.keys() {
        if let Ok((kind, grade)) = multisell_id.to_kind_and_grade() {
            let category_name = kind.category_name();
            category_map
                .entry(category_name)
                .or_default()
                .push((kind, grade));
        }
    }

    let mut categories = Vec::with_capacity(category_map.len());
    for (category_name, kinds_and_grades) in category_map {
        let mut subcategories = kinds_and_grades
            .into_iter()
            .map(|(kind, grade)| NamedMultisell::from((kind, grade)))
            .collect::<Vec<_>>();

        subcategories.sort_by(|a, b| a.grade().cmp(b.grade()));

        if !subcategories.is_empty() {
            categories.push(CategoryMultisell::new(category_name, subcategories));
        }
    }
    categories.sort_by(|a, b| a.name().cmp(b.name()));
    categories
}

const MAX_SUBCATEGORIES_PER_PAGE: usize = 24;

pub(super) fn handle_list(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    admin_query: AdminCommandQuery,
    mut admin_menu: ResMut<AdminMenu>,
    admin_shop: Res<AdminShopMultiSells>,
    characters: Query<Ref<Name>>,
) -> Result<()> {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();

    let (object_id, _) = admin_query.validate_gm(entity)?;

    if let BypassCommand::Admin(AdminMenuCommand::MultiSellList(page)) = cmd {
        let name = characters.get(entity)?;

        let page = *page;
        // Create categories with subcategories for graded items
        let categories = create_multisell_categories(&admin_shop);

        let (item_categories, pagination) = categories.as_slice().paginate_func(
            page.saturating_sub(1) as usize, // Convert from 1-indexed to 0-indexed
            MAX_SUBCATEGORIES_PER_PAGE,
            |category| category.subcategories().len(),
        );

        let mut context = tera::Context::new();
        context.insert("admin_name", &name.to_string());
        context.insert("admin_access_level", &admin_query.account(entity)?.access());
        context.insert("item_categories", &item_categories);
        context.extend(pagination.tera_context());

        let page_name = AdminMenuPage::MultiSellList;
        let rendered = admin_menu.render_with_fallback(page_name.html().as_str(), &context)?;
        admin_menu.set_last_page(entity, page_name, Some(context.clone()));

        commands.trigger_targets(
            GameServerPacket::from(NpcHtmlMessage::new(object_id, rendered, ItemId::default())),
            entity,
        );
        commands.trigger_targets(LastAdminMenuPage, entity);
    }
    Ok(())
}
