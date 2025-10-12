use crate::plugins::npc::drop::GenerateDropPlugin;
use bevy::{log, prelude::*};
use game_core::{
    custom_hierarchy::DespawnChildOf,
    npc::{Bundle as NpcBundle, NpcComponentsPlugin, NpcInfo, Spawn, Spawned},
    object_id::ObjectIdManager,
    spawner::Spawner,
    stats::StatFormulaRegistry,
};
use l2r_core::{
    assets::ASSET_DIR, chronicles::CHRONICLE, model::generic_number::GenericNumber,
    utils::get_base_path,
};
use map::{WorldMap, id::RegionId};

mod commands;
mod dialog;
mod drop;
mod monster_ai;

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NpcComponentsPlugin)
            .add_plugins(GenerateDropPlugin)
            .add_plugins(commands::NpcCommandsPlugin)
            .add_plugins(monster_ai::NpcAiPlugin)
            .add_plugins(dialog::DialogPlugin);

        app.add_observer(spawn_npc_bundle_handler);

        app.add_systems(Update, update_assets);
    }
}

pub fn spawn_npc_bundle_handler(
    spawn: Trigger<Spawn>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    mut spawners: Query<Mut<Spawner>>,
    formula_registry: Res<StatFormulaRegistry>,
    npc_assets: Res<Assets<NpcInfo>>,
    mut object_id_manager: ResMut<ObjectIdManager>,
    asset_server: Res<AssetServer>,
) -> Result<()> {
    let event = spawn.event();
    let spawner_entity = spawn.target();

    let npc_id = event.id;
    let range = npc_id.range();
    let filename = format!("{}.json", range);

    let mut asset_path = get_base_path();
    asset_path.push(ASSET_DIR);
    asset_path.push("npc");
    asset_path.push(CHRONICLE);
    asset_path.push(&filename);

    let new_npc_handle = asset_server.load(asset_path);
    let Some(npc_info) = npc_assets.get(new_npc_handle.id()) else {
        log::warn!("NPC info not found for id: {}", npc_id);
        return Ok(());
    };
    let Some(npc_model) = npc_info.get(&npc_id) else {
        log::warn!("NPC model not found for id: {}", npc_id);
        return Ok(());
    };

    let npc_entity = commands
        .spawn(NpcBundle::new(
            npc_id,
            npc_model.clone(),
            event.transform,
            &formula_registry,
            &mut object_id_manager,
        ))
        .id();
    if let Some(display_id) = npc_model.display_id {
        commands.entity(npc_entity).insert(display_id);
    }

    let region_id = RegionId::from(event.transform.translation);

    if let Some(region_entity) = world_map.get(&region_id).copied() {
        if let Ok(mut spawner) = spawners.get_mut(spawner_entity) {
            if let Some(npc_spawn_info) = spawner.npc_mut(npc_id) {
                npc_spawn_info.inc_count_alive();
            }

            commands
                .entity(npc_entity)
                .insert(DespawnChildOf(spawner_entity));
        } else {
            commands
                .entity(npc_entity)
                .insert(DespawnChildOf(region_entity));
        }
    }

    commands.trigger_targets(Spawned, npc_entity);
    Ok(())
}

fn update_assets(
    mut npc_info: ResMut<Assets<NpcInfo>>,
    mut events: EventReader<AssetEvent<NpcInfo>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && let Some(data) = npc_info.get_strong_handle(*id)
        {
            log::info!("Updated NPC info ({:?})", data.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::chronicles::CHRONICLE;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_npc_from_json() {
        NpcInfo::test_data();
    }

    #[test]
    fn test_parse_all_npc_from_json() {
        let mut npc_dir = get_base_path();
        npc_dir.push(ASSET_DIR);
        npc_dir.push("npc");
        npc_dir.push(CHRONICLE);

        for entry in std::fs::read_dir(npc_dir).expect("Failed to read npc directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file =
                    File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
                let reader = BufReader::new(file);

                let npc_info: NpcInfo = from_reader(reader)
                    .unwrap_or_else(|_| panic!("Failed to parse NPC from JSON: {:?}", path));

                assert!(!npc_info.is_empty());
            }
        }
    }
}
