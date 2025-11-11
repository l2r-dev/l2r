use bevy::{log, prelude::*};
use bevy_asset::LoadedFolder;
use derive_more::derive::From;
use l2r_core::{
    assets::{
        ASSET_DIR,
        html::{HtmlAsset, TeraHtmlTemplater},
    },
    chronicles::CHRONICLE,
};
use state::LoadingSystems;

pub struct DialogComponentsPlugin;
impl Plugin for DialogComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DialogRequest>()
            .register_type::<SendNpcInfoDialog>();

        app.init_resource::<NpcDialogHandles>();

        app.add_systems(
            Update,
            DialogTemplater::init.in_set(LoadingSystems::AssetInit),
        );
    }
}

#[derive(Debug, Event, Reflect)]
pub struct SendNpcInfoDialog(pub Entity);

#[derive(Default, Resource)]
pub struct NpcDialogHandles {
    pub folder: Handle<LoadedFolder>,
    pub htmls: Vec<Handle<HtmlAsset>>,
}

#[derive(Clone, Debug, Resource)]
pub struct DialogTemplater {
    templater: tera::Tera,
}

impl TeraHtmlTemplater for DialogTemplater {
    fn templater(&self) -> &tera::Tera {
        &self.templater
    }

    fn templater_mut(&mut self) -> &mut tera::Tera {
        &mut self.templater
    }
}

impl DialogTemplater {
    pub fn init(world: &mut World) {
        world.init_resource::<Self>();
    }
    /// Renders an NPC dialog using an empty context.
    pub fn npc_dialog(
        &self,
        npc_id: &super::Id,
        kind: &super::Kind,
        page: &str,
    ) -> Result<String, tera::Error> {
        let context = tera::Context::new();
        self.npc_dialog_with_context(npc_id, kind, page, &context)
    }

    /// Renders an NPC dialog with the provided context.
    pub fn npc_dialog_with_context(
        &self,
        npc_id: &super::Id,
        kind: &super::Kind,
        page: &str,
        context: &tera::Context,
    ) -> Result<String, tera::Error> {
        let template_path = Self::build_dialog_path(npc_id, kind, page);
        self.render_with_fallback(&template_path, context)
    }

    /// Constructs the dialog template path based on NPC kind, ID, and page.
    fn build_dialog_path(npc_id: &super::Id, kind: &super::Kind, page: &str) -> String {
        format!(
            "{}/{}/{}.html",
            kind.to_string().to_lowercase(),
            npc_id,
            page
        )
    }

    /// Builds the search path for NPC dialog templates.
    fn build_template_search_path() -> std::path::PathBuf {
        let mut path = l2r_core::utils::get_base_path();
        path.push(ASSET_DIR);
        path.push("html");
        path.push(CHRONICLE);
        path.push("npc");
        path.push("**");
        path.push("*.html");
        path
    }

    fn initialize_templater(template_path: &std::path::Path) -> tera::Tera {
        let path_str = template_path
            .to_str()
            .expect("Template path should be valid UTF-8");

        match tera::Tera::new(path_str) {
            Ok(templater) => templater,
            Err(error) => {
                log::error!(
                    "Failed to load NPC dialog templates from '{}': {}",
                    path_str,
                    error
                );
                tera::Tera::default()
            }
        }
    }

    fn log_loaded_templates(templater: &tera::Tera) {
        let template_count = templater.get_template_names().count();
        log::info!("Loaded {} NPC dialog templates", template_count);

        for name in templater.get_template_names() {
            log::trace!("Loaded NPC template: {}", name);
        }
    }
}

impl Default for DialogTemplater {
    fn default() -> Self {
        let template_path = Self::build_template_search_path();
        let templater = Self::initialize_templater(&template_path);

        Self::log_loaded_templates(&templater);

        Self { templater }
    }
}

#[derive(Component, Deref, From, Reflect)]
#[component(storage = "SparseSet")]
pub struct DialogRequest(pub Entity);
