use bevy::prelude::*;
use bevy_asset::LoadedFolder;
use game_core::{
    admin_menu::{AdminMenu, AdminMenuComponentsPlugin, AdminMenuPagesHandles, LastAdminMenuPage},
    character::Character,
    items,
    network::packets::server::{GameServerPacket, NpcHtmlMessage},
    object_id::ObjectId,
};
use l2r_core::{
    assets::html::{HtmlAsset, TeraHtmlTemplater},
    chronicles::CHRONICLE,
};
use std::path::PathBuf;

mod commands;

pub struct AdminMenuPlugin;
impl Plugin for AdminMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AdminMenuComponentsPlugin);

        app.add_systems(Startup, load_assets);
        app.add_systems(Update, (html_assets_changed, templates_folder_changed));

        app.add_observer(handle_last_admin_menu_page);
        app.add_plugins(commands::AdminMenuCommandsPlugin);
    }
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut admin_menu_handles: ResMut<AdminMenuPagesHandles>,
) {
    let mut asset_dir = PathBuf::new();
    asset_dir.push("html");
    asset_dir.push(CHRONICLE);
    asset_dir.push("admin");

    let loaded_folder = asset_server.load_folder(asset_dir);
    admin_menu_handles.folder = loaded_folder.clone();
}

fn templates_folder_changed(
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    asset_folders: Res<Assets<LoadedFolder>>,
    mut admin_menu_handles: ResMut<AdminMenuPagesHandles>,
    mut admin_menu: ResMut<AdminMenu>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                if admin_menu_handles.folder.id() == *id {
                    let loaded_folder = asset_folders.get(admin_menu_handles.folder.id()).unwrap();

                    admin_menu_handles.htmls = Vec::with_capacity(loaded_folder.handles.len());

                    for handle in loaded_folder.handles.iter() {
                        let typed_handle = handle.clone().typed_unchecked::<HtmlAsset>();
                        admin_menu_handles.htmls.push(typed_handle);
                    }
                    admin_menu.reload();
                }
            }
            _ => {}
        }
    }
}

fn html_assets_changed(
    mut events: EventReader<AssetEvent<HtmlAsset>>,
    admin_menu_handles: Res<AdminMenuPagesHandles>,
    mut admin_menu: ResMut<AdminMenu>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && admin_menu_handles.htmls.iter().any(|h| h.id() == *id)
        {
            admin_menu.reload();
        }
    }
}

fn handle_last_admin_menu_page(
    trigger: Trigger<LastAdminMenuPage>,
    mut commands: Commands,
    admin_menu: Res<AdminMenu>,
    object_ids: Query<Ref<ObjectId>, With<Character>>,
) -> Result<()> {
    let entity = trigger.target();
    let (page, context) = match admin_menu.last_page(entity) {
        Some((page, context)) => (page, context),
        None => return Ok(()),
    };
    let html = admin_menu.render_with_fallback(page.html().as_str(), &context)?;
    let object_id = object_ids.get(entity)?;
    commands.trigger_targets(
        GameServerPacket::from(NpcHtmlMessage::new(*object_id, html, items::Id::default())),
        entity,
    );
    Ok(())
}
