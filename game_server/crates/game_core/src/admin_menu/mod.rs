use crate::{items, multisell, npc, skills, stats::Level, teleport};
use bevy::{log, platform::collections::HashMap, prelude::*};
use bevy_asset::LoadedFolder;
use l2r_core::{
    assets::{
        ASSET_DIR,
        html::{HtmlAsset, TeraHtmlTemplater},
    },
    chronicles::CHRONICLE,
};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

pub struct AdminMenuComponentsPlugin;
impl Plugin for AdminMenuComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AdminMenuPage>();

        app.init_resource::<AdminMenuPagesHandles>();
        app.init_resource::<AdminMenu>();
    }
}

#[derive(Default, Resource)]
pub struct AdminMenuPagesHandles {
    pub folder: Handle<LoadedFolder>,
    pub htmls: Vec<Handle<HtmlAsset>>,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, EnumString, AsRefStr, EnumIter, Display,
)]
pub enum AdminMenuPage {
    MainMenu,
    TpList,
    MultiSellList,
}

impl AdminMenuPage {
    pub fn html(&self) -> String {
        format!("{self}.html")
    }
}

#[derive(Clone, Debug, Event)]
pub struct LastAdminMenuPage;

#[derive(Clone, Debug, Resource)]
pub struct AdminMenu {
    templater: tera::Tera,
    last_page: HashMap<Entity, (AdminMenuPage, Option<tera::Context>)>,
}

impl AdminMenu {
    pub fn last_page(&self, entity: Entity) -> Option<(AdminMenuPage, tera::Context)> {
        self.last_page.get(&entity).cloned().map(|(page, context)| {
            let context = context.unwrap_or_else(tera::Context::new);
            (page, context)
        })
    }

    pub fn set_last_page(
        &mut self,
        entity: Entity,
        page: AdminMenuPage,
        context: Option<tera::Context>,
    ) {
        self.last_page.insert(entity, (page, context));
    }
}

impl TeraHtmlTemplater for AdminMenu {
    const DEFAULT_TEMPLATE_PATH: &'static str = "MainMenu.html";
    fn templater(&self) -> &tera::Tera {
        &self.templater
    }

    fn templater_mut(&mut self) -> &mut tera::Tera {
        &mut self.templater
    }
}

impl Default for AdminMenu {
    fn default() -> Self {
        let mut base_dir = l2r_core::utils::get_base_path();
        base_dir.push(ASSET_DIR);
        base_dir.push("html");
        base_dir.push(CHRONICLE);
        base_dir.push("admin");
        base_dir.push("**");
        base_dir.push("*.html");

        let tera = match tera::Tera::new(base_dir.to_str().unwrap()) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to load admin menu templates: {}", e);
                tera::Tera::default()
            }
        };
        Self {
            templater: tera,
            last_page: HashMap::new(),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumDiscriminants, Eq, Hash, PartialEq, Reflect,
)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(name(CommandVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum AdminMenuCommand {
    Main,
    Heal,
    Resurrect,
    Kill,
    Pause,
    SpawnItem(items::Id, u64),
    AddSkill(skills::Id, skills::Level),
    SpawnNpc(npc::Id),
    Tp(teleport::Id),
    TpList(u32),
    MultiSell(multisell::Id),
    MultiSellList(u32),
    SetLevel(Level),
}

impl FromStr for AdminMenuCommand {
    type Err = String;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let mut parts = command.splitn(2, ' ');
        let base_command = parts.next().unwrap_or("");
        let arg = parts.next();

        let variant = CommandVariants::from_str(base_command)
            .map_err(|_| format!("Unknown admin command: {base_command}"))?;

        match variant {
            CommandVariants::Main => Ok(AdminMenuCommand::Main),
            CommandVariants::Heal => Ok(AdminMenuCommand::Heal),
            CommandVariants::Resurrect => Ok(AdminMenuCommand::Resurrect),
            CommandVariants::Kill => Ok(AdminMenuCommand::Kill),
            CommandVariants::Pause => Ok(AdminMenuCommand::Pause),
            CommandVariants::SpawnItem => {
                if let Some(arg) = arg {
                    // Space-separated the item id and count
                    let mut item_parts = arg.splitn(2, ' ');
                    if let Some(item_id_str) = item_parts.next() {
                        let item_id = item_id_str
                            .parse::<items::Id>()
                            .map_err(|_| format!("Invalid item ID: {item_id_str}"))?;

                        let count = item_parts
                            .next()
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(1);

                        return Ok(AdminMenuCommand::SpawnItem(item_id, count));
                    }
                }
                Err(format!(
                    "Invalid or missing argument for spawn_item: {command}"
                ))
            }
            CommandVariants::AddSkill => {
                if let Some(arg) = arg {
                    // Space-separated the skill id and level
                    let mut skill_parts = arg.splitn(2, ' ');
                    if let Some(skill_id_str) = skill_parts.next() {
                        let skill_id = skill_id_str
                            .parse::<skills::Id>()
                            .map_err(|_| format!("Invalid skill ID: {skill_id_str}"))?;

                        let level = skill_parts
                            .next()
                            .and_then(|s| s.parse::<skills::Level>().ok())
                            .unwrap_or(1.into());

                        return Ok(AdminMenuCommand::AddSkill(skill_id, level));
                    }
                }
                Err(format!(
                    "Invalid or missing argument for add_skill: {command}"
                ))
            }
            CommandVariants::SpawnNpc => {
                if let Some(arg) = arg
                    && let Ok(npc_id) = arg.parse::<npc::Id>()
                {
                    return Ok(AdminMenuCommand::SpawnNpc(npc_id));
                }
                Err(format!(
                    "Invalid or missing argument for spawn_npc: {command}"
                ))
            }
            CommandVariants::Tp => {
                if let Some(arg) = arg {
                    return arg
                        .parse::<teleport::Id>()
                        .map(AdminMenuCommand::Tp)
                        .map_err(|_| format!("Invalid argument for teleport: {arg}"));
                }
                Err(format!(
                    "Invalid or missing argument for teleport: {command}, args: {arg:?}"
                ))
            }
            CommandVariants::TpList => {
                let page = if let Some(page_str) = arg {
                    if page_str.trim().is_empty() {
                        1
                    } else {
                        page_str.parse::<u32>().unwrap_or(1)
                    }
                } else {
                    1
                };
                Ok(AdminMenuCommand::TpList(page))
            }
            CommandVariants::MultiSell => {
                if let Some(arg) = arg {
                    return arg
                        .parse::<multisell::Id>()
                        .map(AdminMenuCommand::MultiSell)
                        .map_err(|_| format!("Invalid argument for multisell: {arg}"));
                }
                Err(format!(
                    "Invalid or missing argument for multisell: {command}, args: {arg:?}"
                ))
            }
            CommandVariants::MultiSellList => {
                let page = if let Some(page_str) = arg {
                    if page_str.trim().is_empty() {
                        1
                    } else {
                        page_str.parse::<u32>().unwrap_or(1)
                    }
                } else {
                    1
                };
                Ok(AdminMenuCommand::MultiSellList(page))
            }
            CommandVariants::SetLevel => {
                if let Some(arg) = arg {
                    return arg
                        .parse::<Level>()
                        .map(AdminMenuCommand::SetLevel)
                        .map_err(|_| format!("Invalid level argument: {arg}"));
                }
                Err(format!(
                    "Invalid or missing argument for set_level: {command}"
                ))
            }
        }
    }
}
