use bevy::prelude::*;
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

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub database_url: DatabaseUrl,
    pub redis_url: RedisUrl,
}

#[derive(Asset, Clone, Debug, Default, Deserialize, Reflect, Resource, Serialize)]
#[reflect(Resource)]
#[serde(default)]
pub struct Config {
    general: GeneralConfig,
    #[serde(skip)]
    #[reflect(ignore)]
    handle: Handle<Config>,
}

impl Config {
    /// Load config from file with defaults and environment overrides
    fn load() -> Self {
        let mut config = Self::default();
        let config_path = get_base_path().join(ASSET_DIR).join(CONFIG_FILE);

        if let Ok(config_content) = std::fs::read_to_string(&config_path) {
            if let Ok(file_config) = toml::from_str::<Config>(&config_content) {
                config.merge(&file_config);
            } else {
                warn!("Failed to parse config file, using defaults");
            }
        } else {
            warn!("Config file not found at {:?}, using defaults", config_path);
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

    /// Merge another config into self, overriding fields if present in other.
    fn merge(&mut self, other: &Config) {
        // General
        self.general.database_url = other.general.database_url.clone();
        self.general.redis_url = other.general.redis_url.clone();
    }

    /// Override fields present in env vars.
    fn apply_env_overrides(&mut self) {
        for (key, value) in std::env::vars().filter(|(key, _)| key.starts_with(ENV_PREFIX)) {
            match key.trim_start_matches(ENV_PREFIX).to_uppercase().as_str() {
                "DATABASE_URL" => self.general.database_url = value.into(),
                "REDIS_URL" => self.general.redis_url = value.into(),
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
            let id = config.handle.id();
            if event.is_loaded_with_dependencies(id)
                && let Some(asset_config) = config_assets.get(id)
            {
                // Start with defaults
                let mut merged = Config::default();
                // Merge config file over defaults
                merged.merge(asset_config);
                // Merge env vars over that
                merged.apply_env_overrides();
                merged.handle = config.handle.clone();
                *config = merged;
            }
        }
    }
}
