use bevy::{log, prelude::*};
use bevy_common_assets::toml::TomlAssetPlugin;
use derive_more::derive::{From, Into};
use l2r_core::{assets::ASSET_DIR, utils::get_base_path};
use serde::{Deserialize, Serialize};
pub struct ConfigPlugin;

const CONFIG_FILE: &str = "config.toml";
const ENV_PREFIX: &str = "L2R_";

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Config>();

        app.insert_resource(Config::load());

        app.add_plugins(TomlAssetPlugin::<Config>::new(&[CONFIG_FILE]));

        app.add_systems(Startup, Config::initial_load)
            .add_systems(Update, Config::update_from_asset);
    }
}

#[derive(Clone, Debug, Deserialize, From, Into, Reflect, Serialize)]
pub struct DatabaseUrl(pub String);
impl Default for DatabaseUrl {
    fn default() -> Self {
        Self("postgresql://l2r:l2r@localhost/l2r".to_string())
    }
}

#[derive(Clone, Debug, Deserialize, From, Into, Reflect, Serialize)]
pub struct RedisUrl(pub String);
impl Default for RedisUrl {
    fn default() -> Self {
        Self("redis://localhost:6379/0".to_string())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, From, Into, Reflect, Serialize)]
pub struct MaxPlayers(usize);
impl Default for MaxPlayers {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub max_players: MaxPlayers,
    pub database_url: DatabaseUrl,
    pub redis_url: RedisUrl,
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct SkillsConfig {
    pub not_used_yet_parm: f32,
}
impl Default for SkillsConfig {
    fn default() -> Self {
        Self {
            not_used_yet_parm: 0.95,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
#[serde(default)]
pub struct GameplayConfig {
    pub free_teleports: bool,
    pub regen_rate: f32,
    pub max_crit_rate: u32,
    pub min_npc_level_dmg_penalty: u32,
    pub npc_dmg_penalty: Vec<f32>,
    pub npc_crit_dmg_penalty: Vec<f32>,
    pub npc_skill_dmg_penalty: Vec<f32>,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            free_teleports: false,
            regen_rate: 1.0,
            max_crit_rate: 500,
            min_npc_level_dmg_penalty: 78,
            npc_dmg_penalty: vec![0.7, 0.6, 0.6, 0.55],
            npc_crit_dmg_penalty: vec![0.75, 0.65, 0.6, 0.58],
            npc_skill_dmg_penalty: vec![0.8, 0.7, 0.65, 0.62],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct GuiConfig {
    pub geodata_cells: bool,
    pub geodata_blocks: bool,
}

#[derive(Asset, Clone, Debug, Default, Deserialize, Reflect, Resource, Serialize)]
#[reflect(Resource)]
#[serde(default)]
pub struct Config {
    general: GeneralConfig,
    skills: SkillsConfig,
    gameplay: GameplayConfig,
    gui: GuiConfig,
    #[serde(skip)]
    #[reflect(ignore)]
    handle: Handle<Config>,
}

impl Config {
    fn load() -> Self {
        let mut config = Self::default();
        let config_path = get_base_path().join(ASSET_DIR).join(CONFIG_FILE);

        if let Ok(config_content) = std::fs::read_to_string(&config_path) {
            if let Ok(file_config) = toml::from_str::<Config>(&config_content) {
                config.merge(&file_config);
            } else {
                log::warn!("Failed to parse config file, using defaults");
            }
        } else {
            log::warn!("Config file not found at {:?}, using defaults", config_path);
        }

        config.apply_env_overrides();
        config
    }

    fn initial_load(mut config: ResMut<Config>, asset_server: Res<AssetServer>) {
        let handle = asset_server.load(CONFIG_FILE);
        config.handle = handle;
        config.apply_env_overrides();
    }

    pub fn general(&self) -> &GeneralConfig {
        &self.general
    }

    pub fn skills(&self) -> &SkillsConfig {
        &self.skills
    }

    pub fn gameplay(&self) -> &GameplayConfig {
        &self.gameplay
    }

    pub fn gui(&self) -> &GuiConfig {
        &self.gui
    }

    /// Merge another config into self, overriding fields if present in other.
    fn merge(&mut self, other: &Config) {
        // General
        self.general.max_players = other.general.max_players;
        self.general.database_url = other.general.database_url.clone();
        self.general.redis_url = other.general.redis_url.clone();
        // Skills
        self.skills.not_used_yet_parm = other.skills.not_used_yet_parm;
        // Gameplay
        self.gameplay.free_teleports = other.gameplay.free_teleports;
        self.gameplay.regen_rate = other.gameplay.regen_rate;
        self.gameplay.max_crit_rate = other.gameplay.max_crit_rate;
        self.gameplay.min_npc_level_dmg_penalty = other.gameplay.min_npc_level_dmg_penalty;
        self.gameplay.npc_dmg_penalty = other.gameplay.npc_dmg_penalty.clone();
        self.gameplay.npc_crit_dmg_penalty = other.gameplay.npc_crit_dmg_penalty.clone();
        self.gameplay.npc_skill_dmg_penalty = other.gameplay.npc_skill_dmg_penalty.clone();
        // GUI
        self.gui.geodata_cells = other.gui.geodata_cells;
        self.gui.geodata_blocks = other.gui.geodata_blocks;
    }

    /// Override fields present in env vars.
    fn apply_env_overrides(&mut self) {
        // First, handle the unprefixed DATABASE_URL if present
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.general.database_url = db_url.into();
        }
        // Handle the unprefixed REDIS_URL if present
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            self.general.redis_url = redis_url.into();
        }
        for (key, value) in std::env::vars().filter(|(key, _)| key.starts_with(ENV_PREFIX)) {
            match key.trim_start_matches(ENV_PREFIX).to_uppercase().as_str() {
                "MAX_PLAYERS" => {
                    self.general.max_players =
                        value.parse::<usize>().map(MaxPlayers).unwrap_or_default()
                }
                "DATABASE_URL" => self.general.database_url = value.into(),
                "REDIS_URL" => self.general.redis_url = value.into(),
                "NOT_USED_YET_PARM" => {
                    self.skills.not_used_yet_parm = value
                        .parse::<f32>()
                        .unwrap_or(self.skills.not_used_yet_parm)
                }
                "FREE_TELEPORTS" => {
                    self.gameplay.free_teleports = value
                        .parse::<bool>()
                        .unwrap_or(self.gameplay.free_teleports)
                }
                "REGEN_RATE" => {
                    self.gameplay.regen_rate =
                        value.parse::<f32>().unwrap_or(self.gameplay.regen_rate)
                }
                "MAX_CRIT_RATE" => {
                    self.gameplay.max_crit_rate =
                        value.parse::<u32>().unwrap_or(self.gameplay.max_crit_rate)
                }
                "MIN_NPC_LEVEL_DMG_PENALTY" => {
                    self.gameplay.min_npc_level_dmg_penalty = value
                        .parse::<u32>()
                        .unwrap_or(self.gameplay.min_npc_level_dmg_penalty)
                }
                "NPC_DMG_PENALTY" => {
                    let parsed_vec: Result<Vec<f32>, _> =
                        value.split(',').map(|s| s.trim().parse::<f32>()).collect();
                    if let Ok(vec) = parsed_vec {
                        self.gameplay.npc_dmg_penalty = vec;
                    }
                }
                "NPC_CRIT_DMG_PENALTY" => {
                    let parsed_vec: Result<Vec<f32>, _> =
                        value.split(',').map(|s| s.trim().parse::<f32>()).collect();
                    if let Ok(vec) = parsed_vec {
                        self.gameplay.npc_crit_dmg_penalty = vec;
                    }
                }
                "NPC_SKILL_DMG_PENALTY" => {
                    let parsed_vec: Result<Vec<f32>, _> =
                        value.split(',').map(|s| s.trim().parse::<f32>()).collect();
                    if let Ok(vec) = parsed_vec {
                        self.gameplay.npc_skill_dmg_penalty = vec;
                    }
                }
                "GUI_GEODATA_CELLS" => {
                    self.gui.geodata_cells = value.parse::<bool>().unwrap_or(self.gui.geodata_cells)
                }
                "GUI_GEODATA_BLOCKS" => {
                    self.gui.geodata_blocks =
                        value.parse::<bool>().unwrap_or(self.gui.geodata_blocks)
                }
                _ => {}
            }
        }
    }

    fn update_from_asset(
        config_assets: Res<Assets<Config>>,
        mut events: EventReader<AssetEvent<Config>>,
        mut config: ResMut<Config>,
    ) {
        for event in events.read() {
            match event {
                AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                    let handle = config.handle.clone();
                    if handle.id() != *id {
                        log::error!(
                            "Config asset loaded with id: {:?} 
                            does not match the current config handle id: {:?},
                            we expected single config asset",
                            id,
                            handle.id()
                        );
                        continue;
                    }

                    if let Some(asset_config) = config_assets.get(*id) {
                        // Start with defaults
                        let mut merged = Config::default();
                        // Merge config file over defaults
                        merged.merge(asset_config);
                        // Merge env vars over that
                        merged.apply_env_overrides();
                        merged.handle = handle;
                        *config = merged;
                    }
                }
                _ => {}
            }
        }
    }
}
