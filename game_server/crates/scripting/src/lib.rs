use bevy::{prelude::*, reflect::TypeRegistration};
pub use bevy_mod_scripting::*;
use bevy_mod_scripting::{
    bindings::InteropError,
    core::{BMSScriptingInfrastructurePlugin, script::*},
    lua::{LuaContext, LuaScriptingPlugin},
    prelude::{CoreScriptGlobalsPlugin, GlobalNamespace, NamespaceBuilder},
};
use l2r_core::utils::get_base_path;

pub mod runtime;
pub mod utils;

pub use runtime::*;

const PACKAGE_NAME: &str = "package";
const PATH_NAME: &str = "path";
const LUA_PATTERN: &str = "?.lua";

pub struct CustomScriptingPlugin;

impl Plugin for CustomScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Configure lua context to include base path for scripts
        let mut lua_scripting = LuaScriptingPlugin::default();
        let base_path_configurator = |_attachment: &ScriptAttachment, context: &mut LuaContext| {
            let package: lua::mlua::Table = context
                .globals()
                .get(PACKAGE_NAME)
                .map_err(|e| InteropError::external(Box::new(e)))?;
            let current_path: String = package
                .get(PATH_NAME)
                .map_err(|e| InteropError::external(Box::new(e)))?;
            let custom_path = format!(
                "{}{}{};{}",
                get_base_path().display(),
                std::path::MAIN_SEPARATOR,
                LUA_PATTERN,
                current_path
            );
            package
                .set(PATH_NAME, custom_path)
                .map_err(|e| InteropError::external(Box::new(e)))?;
            Ok(())
        };

        lua_scripting
            .scripting_plugin
            .context_initializers
            .push(base_path_configurator);

        // Filter to exclude duplicate type registrations
        // This prevents warning messages about duplicate entries in the types global
        // These short type names appear in multiple modules causing conflicts
        // They still can be accessed via world.get_type_by_name with full path
        fn type_filter(registration: &TypeRegistration) -> bool {
            let type_path = registration.type_info().type_path_table().short_path();
            !matches!(
                type_path,
                "Kind"              // skills::Kind, npc::Kind
                | "Model"           // items::Model, shortcut::Model, npc::Model, character::Model, etc.
                | "Id"              // skills::Id, items::Id, npc::Id, teleport::Id, multisell::Id
                | "Level"           // skills::Level, stats::progress::Level
                | "Weapon"          // items::kind::Weapon, stats::attack::effect_kind::Weapon
                | "ScalingMode"     // Bevy camera ScalingMode (multiple camera types)
                | "Option<Id>"      // Option<skills::Id>, Option<items::Id>, etc.
                | "Range<f64>"      // std::ops::Range<f64> - multiple registrations
                | "Range<u32>"      // std::ops::Range<u32> - multiple registrations
                | "RangeInclusive<f64>" // std::ops::RangeInclusive<f64> - multiple registrations
            )
        }

        app.add_plugins(ScriptFunctionsPlugin)
            .add_plugins(CoreScriptGlobalsPlugin {
                filter: type_filter,
                ..Default::default()
            })
            .add_plugins(BMSScriptingInfrastructurePlugin::default())
            .add_plugins(lua_scripting)
            .add_plugins(RuntimeScriptPlugin);

        app.register_type::<ScriptComponent>();

        l2r_core::register_optional_types!(app, Vec3, Entity, Timer, i32);
        Self::register_script_functions(app);
    }
}

impl CustomScriptingPlugin {
    fn register_script_functions(app: &mut App) -> &mut App {
        let world = app.world_mut();
        let mut namespace = NamespaceBuilder::<GlobalNamespace>::new_unregistered(world);

        Self::register_bevy_log(&mut namespace);
        namespace.register("rand", rand::random::<f32>);

        app
    }

    fn register_bevy_log(namespace: &mut NamespaceBuilder<GlobalNamespace>) {
        namespace.register("bevy_log", |args: Vec<String>| {
            if args.is_empty() {
                bevy::log::warn!("bevy_log called with empty arguments");
                return;
            }

            let level = &args[0];
            let message = if args.len() > 1 {
                args[1..].join(" ")
            } else {
                String::new()
            };

            match level.as_str() {
                "trace" => bevy::log::trace!("{}", message),
                "debug" => bevy::log::debug!("{}", message),
                "info" => bevy::log::info!("{}", message),
                "warn" => bevy::log::warn!("{}", message),
                "error" => bevy::log::error!("{}", message),
                _ => bevy::log::warn!("Unknown log level '{}': {}", level, message),
            }
        });
    }
}
