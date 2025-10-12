use bevy::{
    asset::{Assets, LoadedFolder},
    log,
    prelude::*,
};
use bevy_mod_scripting::{
    asset::ScriptAsset,
    core::{commands::AttachScript, script::ScriptAttachment},
    lua::LuaScriptingPlugin,
};
use state::LoadingSystems;
use std::path::PathBuf;

#[derive(Debug, Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct RuntimeScripts {
    pub folder: Handle<LoadedFolder>,
    pub scripts: Vec<Handle<ScriptAsset>>,
}

#[derive(Clone, Component, Copy, Default)]
pub struct RuntimeScriptsTaskSpawned {
    pub loaded: bool,
}

pub struct RuntimeScriptPlugin;

impl Plugin for RuntimeScriptPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RuntimeScripts>();

        app.init_resource::<RuntimeScripts>();

        app.add_systems(
            Update,
            RuntimeScriptsManager::init.in_set(LoadingSystems::RuntimeScriptsInit),
        )
        .add_systems(Update, RuntimeScriptsManager::folder_changed);
    }
}

struct RuntimeScriptsManager;
impl RuntimeScriptsManager {
    const SCRIPT_DIR: &str = "scripts";
    const RUNTIME_DIR: &str = "runtime";
    const ENTRY_SCRIPT: &str = "app.lua";

    pub fn init(
        mut commands: Commands,
        task_flag: Query<(Entity, Ref<RuntimeScriptsTaskSpawned>)>,

        asset_server: Res<AssetServer>,
        mut runtime_scripts: ResMut<RuntimeScripts>,
    ) {
        if let Ok((entity, scripts_task_flag)) = task_flag.single() {
            if scripts_task_flag.loaded {
                commands.entity(entity).despawn();
            }
        } else {
            commands.spawn(RuntimeScriptsTaskSpawned::default());
            Self::load_scripts(&asset_server, &mut runtime_scripts);
        }
    }

    fn load_scripts(asset_server: &AssetServer, runtime_scripts: &mut RuntimeScripts) {
        let mut scripts_dir = PathBuf::new();
        scripts_dir.push(Self::SCRIPT_DIR);
        scripts_dir.push(Self::RUNTIME_DIR);

        log::info!("Loading runtime scripts from: {:?}", scripts_dir);
        let loaded_folder = asset_server.load_folder(scripts_dir);
        runtime_scripts.folder = loaded_folder;
    }

    fn folder_changed(
        mut events: EventReader<AssetEvent<LoadedFolder>>,
        asset_folders: Res<Assets<LoadedFolder>>,
        mut runtime_scripts: ResMut<RuntimeScripts>,
        mut commands: Commands,
        mut task_flag: Query<&mut RuntimeScriptsTaskSpawned>,
    ) {
        for event in events.read() {
            match event {
                AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                    if runtime_scripts.folder.id() == *id {
                        let loaded_folder = asset_folders.get(runtime_scripts.folder.id()).unwrap();
                        runtime_scripts.scripts.clear();
                        runtime_scripts.scripts.reserve(loaded_folder.handles.len());

                        for handle in loaded_folder.handles.iter() {
                            if let Some(path) = handle.path() {
                                if path
                                    .path()
                                    .to_string_lossy()
                                    .ends_with(RuntimeScriptsManager::ENTRY_SCRIPT)
                                {
                                    let script_handle =
                                        handle.clone().typed_unchecked::<ScriptAsset>();
                                    runtime_scripts.scripts.push(script_handle.clone());
                                    log::info!("Adding runtime script: {}", path);
                                    commands.queue(AttachScript::<LuaScriptingPlugin>::new(
                                        ScriptAttachment::StaticScript(script_handle),
                                    ));
                                }
                            }
                        }

                        // Mark the task as loaded when all scripts are processed
                        if let Ok(mut task) = task_flag.single_mut() {
                            task.loaded = true;
                            log::info!("Runtime scripts loading completed");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
