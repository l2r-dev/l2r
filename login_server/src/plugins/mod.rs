use crate::plugins::accounts::AccountsPlugin;
use bevy::{
    app::{PluginGroupBuilder, ScheduleRunnerPlugin},
    prelude::*,
};
use l2r_core::{assets::CustomAssetPlugin, metrics::MetricsPlugin};
use std::time::Duration;

pub mod accounts;
pub mod config;
pub mod db;
pub mod network;
pub mod server_manager;
pub mod state;

pub const FIXED_UPDATE_TICK_RATE: f64 = 60.0;
pub const UPDATE_TICK_RATE: f64 = 30.0;

pub struct ServerNonUiPlugin;
impl Plugin for ServerNonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::log::LogPlugin::default())
            .insert_resource(Time::<Fixed>::from_hz(FIXED_UPDATE_TICK_RATE))
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / UPDATE_TICK_RATE),
            )))
            .add_plugins(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(TransformPlugin)
            .add_plugins(CustomAssetPlugin::custom());
    }
}

pub struct Core;
impl PluginGroup for Core {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ServerNonUiPlugin)
            .add(state::LoginStateSystemsPlugin)
            .add(config::ConfigPlugin)
            // MetricsPlugin must come before plugins that register metrics
            // It also adds bevy_webgate which internally adds AsyncPlugin
            .add(MetricsPlugin::new("loginserver_".to_string(), 8032))
            .add(db::LoginDbPlugin)
            .add(AccountsPlugin)
            .add(network::NetworkPlugin)
            .add(server_manager::ServerManagerPlugin)
    }

    fn name() -> String {
        core::any::type_name::<Self>().to_string()
    }

    fn set<T: Plugin>(self, plugin: T) -> bevy::app::PluginGroupBuilder {
        self.build().set(plugin)
    }
}
