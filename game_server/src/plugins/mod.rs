mod abnormal_effects;
mod action;
mod active_action;
mod admin_menu;
mod attack;
mod auth;
mod character;
mod chat;
mod custom_hierarchy;
pub mod db;
mod doors;
mod encounters;
#[cfg(feature = "gui")]
mod gui;
mod items;
mod manor;
mod movement;
mod multisell;
mod network;
mod npc;
mod object_id;
mod player_specific;
mod shortcuts;
mod shutdown;
mod skills;
mod state;
mod stats;
mod teleport;
mod world_map;

use crate::plugins::state::GameStateProcessPlugin;
use avian3d::PhysicsPlugins;
use bevy::{
    app::{PluginGroupBuilder, ScheduleRunnerPlugin},
    diagnostic::DiagnosticsPlugin,
    log::LogPlugin,
    prelude::*,
    state::app::StatesPlugin,
};
use game_core::consts::{FIXED_UPDATE_TICK_RATE, UPDATE_TICK_RATE};
#[cfg(feature = "gui")]
use gui::ServerUIPlugin;
use l2r_core::{
    assets::{CustomAssetPlugin, CustomAssetWatcherPlugin},
    metrics::MetricsPlugin,
};
use std::time::Duration;
use system_messages::SystemMessagesPlugin;

pub struct ServerNonUiPlugin;
impl Plugin for ServerNonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LogPlugin::default())
            .insert_resource(Time::<Fixed>::from_hz(FIXED_UPDATE_TICK_RATE))
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / UPDATE_TICK_RATE),
            )))
            .add_plugins(DiagnosticsPlugin)
            .add_plugins(StatesPlugin)
            .add_plugins(TransformPlugin)
            .add_plugins(CustomAssetPlugin::custom());
    }
}

pub struct UiSwitchPlugin;
impl Plugin for UiSwitchPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "gui")]
        app.add_plugins(ServerUIPlugin);
        #[cfg(not(feature = "gui"))]
        app.add_plugins(ServerNonUiPlugin);
    }
}

pub struct Core;
impl PluginGroup for Core {
    fn build(self) -> PluginGroupBuilder {
        // Suppress warnings about unused mut, due to feature flags
        #[allow(unused_mut)]
        let mut builder = PluginGroupBuilder::start::<Self>()
            .add(CustomAssetWatcherPlugin)
            .add_group(PhysicsPlugins::default())
            .add(UiSwitchPlugin)
            .add(custom_hierarchy::CustomHierarchyPlugin)
            .add(config::ConfigPlugin)
            .add(GameStateProcessPlugin)
            .add(auth::AuthPlugin)
            .add(world_map::WorldMapPlugin)
            .add(object_id::ObjectIdPlugin)
            .add(l2r_core::assets::html::HtmlPlugin)
            .add(admin_menu::AdminMenuPlugin)
            .add(SystemMessagesPlugin)
            .add(stats::StatsPlugin)
            .add(active_action::ActiveActionPlugin)
            .add(character::CharacterPlugin)
            .add(npc::NpcPlugin)
            .add(movement::MovementPlugin)
            .add(attack::AttackPlugin)
            .add(skills::SkillsPlugin)
            .add(abnormal_effects::AbnormalEffectsPlugin)
            .add(encounters::EncountersPlugin)
            .add(chat::ChatPlugin)
            .add(teleport::TeleportPlugin)
            .add(action::UseActionPlugin)
            .add(items::ItemsPlugin)
            .add(multisell::MultisellPlugin)
            .add(shortcuts::ShortcutPlugin)
            .add(player_specific::PlayerSpecificPlugin)
            .add(doors::DoorsPlugin)
            .add(manor::ManorPlugin);
        {
            builder = builder.add(scripting::CustomScriptingPlugin);
        }
        builder
    }
}

pub struct CoreWithInfrastructure;
impl PluginGroup for CoreWithInfrastructure {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // MetricsPlugin must come first because:
            // 1. It adds bevy_webgate which internally adds AsyncPlugin
            // 2. It creates the Metrics resource that Core plugins need
            .add(MetricsPlugin::new("gameserver_".to_string(), 8031))
            .add(shutdown::ShutdownPlugin)
            .add_group(Core)
            .add(db::GameDbPlugin)
            .add(network::NetworkPlugin)
    }
}
