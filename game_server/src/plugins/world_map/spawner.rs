use avian3d::prelude::*;
use bevy::{ecs::system::ParallelCommands, log, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use game_core::{
    custom_hierarchy::{DespawnChildOf, DespawnChildren},
    npc::{self, RegionalNpcHandles},
    spawner::{
        BannedSpawnZone, NpcSpawnInfo, RegionalSpawners, SpawnList, SpawnListHandle, SpawnZone,
        Spawner,
    },
};
use l2r_core::{
    assets::ASSET_DIR, chronicles::CHRONICLE, model::generic_number::GenericNumber,
    utils::get_base_path,
};
use map::{
    Region, RegionGeoData, WorldMapQuery, ZoneKind, collider::RandomPointWithGeodata, id::RegionId,
};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<SpawnList>::new(&["json"]));

        app.register_type::<Spawner>()
            .register_type::<NpcSpawnInfo>();

        app.add_systems(Update, spawn_npc_spawners);
        app.add_systems(Update, spawn_npc_on_spawner);
    }
}

fn spawn_npc_spawners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spawn_lists: Res<Assets<SpawnList>>,
    mut events: EventReader<AssetEvent<SpawnList>>,
    mut regions: Query<(
        Ref<Region>,
        Ref<SpawnListHandle>,
        Mut<RegionalNpcHandles>,
        Ref<DespawnChildren>,
    )>,
    mut regional_spawners: Query<Entity, With<RegionalSpawners>>,
    mut spawners: Query<Entity, With<Spawner>>,
    mut npcs: Query<Entity, With<npc::Kind>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                handle_spawn_list_loaded(
                    commands.reborrow(),
                    &asset_server,
                    &spawn_lists,
                    *id,
                    regions.reborrow(),
                    regional_spawners.reborrow(),
                    spawners.reborrow(),
                    npcs.reborrow(),
                );
            }
            _ => {}
        }
    }
}

fn handle_spawn_list_loaded(
    mut commands: Commands,
    asset_server: &AssetServer,
    spawn_lists: &Assets<SpawnList>,
    asset_id: AssetId<SpawnList>,
    mut regions: Query<(
        Ref<Region>,
        Ref<SpawnListHandle>,
        Mut<RegionalNpcHandles>,
        Ref<DespawnChildren>,
    )>,
    mut regional_spawners: Query<Entity, With<RegionalSpawners>>,
    mut spawners: Query<Entity, With<Spawner>>,
    mut npcs: Query<Entity, With<npc::Kind>>,
) {
    for (region, spawn_handle, mut regional_npc_handles, despawn_children) in regions.iter_mut() {
        if asset_id == spawn_handle.id()
            && let Some(spawn_list) = spawn_lists.get(asset_id)
        {
            let region_id = region.id();
            log::debug!("Spawn list for region {} loaded", region_id);

            let mut asset_dir = get_base_path();
            asset_dir.push(ASSET_DIR);
            asset_dir.push("npc");
            asset_dir.push(CHRONICLE);

            for spawner in spawn_list.iter() {
                for npc in spawner.npcs.iter() {
                    if !regional_npc_handles.contains_key(&npc.id()) {
                        let filename = format!("{}.json", npc.id().range());

                        let mut asset_path = asset_dir.clone();
                        asset_path.push(&filename);

                        let new_npc_handle = asset_server.load(asset_path);
                        regional_npc_handles.insert(npc.id(), new_npc_handle);
                    }
                }
            }

            process_region_spawns(
                commands.reborrow(),
                spawn_list,
                region_id,
                &despawn_children,
                regional_spawners.reborrow(),
                spawners.reborrow(),
                npcs.reborrow(),
            );
        }
    }
}

fn process_region_spawns(
    mut commands: Commands,
    spawn_list: &SpawnList,
    region_id: RegionId,
    region_children: &DespawnChildren,
    regional_spawners: Query<Entity, With<RegionalSpawners>>,
    spawners: Query<Entity, With<Spawner>>,
    npcs: Query<Entity, With<npc::Kind>>,
) {
    let Some(regional_spawners_entity) = region_children
        .iter()
        .find(|child| regional_spawners.get(*child).is_ok())
    else {
        return;
    };

    // Despawn existing for hot-reloading
    for child in region_children.iter() {
        if spawners.get(child).is_ok() || npcs.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
    spawn_new_spawners(commands, spawn_list, region_id, regional_spawners_entity);
}

fn spawn_new_spawners(
    mut commands: Commands,
    spawn_list: &SpawnList,
    region_id: RegionId,
    regional_spawners_entity: Entity,
) {
    for spawner in spawn_list.iter() {
        let built_spawner = spawner.build();
        let spawner_entity = commands.spawn(built_spawner.clone()).id();
        commands
            .entity(spawner_entity)
            .insert(DespawnChildOf(regional_spawners_entity));

        if let Some(zone) = &built_spawner.0.zone {
            spawn_zones_for_spawner(
                commands.reborrow(),
                zone,
                region_id,
                &spawner.name,
                spawner_entity,
            );
        }
    }
}

fn spawn_zones_for_spawner(
    mut commands: Commands,
    zone: &map::Zone,
    region_id: RegionId,
    spawner_name: &Option<String>,
    spawner_entity: Entity,
) {
    let ZoneKind::Spawn(spawn_zone) = zone.kind() else {
        log::error!("Zone kind is not Spawn");
        return;
    };

    // Spawn banned zone if it exists
    if let Some(banned_zone) = spawn_zone.banned_zone() {
        spawn_banned_zone(
            commands.reborrow(),
            banned_zone,
            region_id,
            spawner_name,
            spawner_entity,
        );
    }

    // Spawn main zone
    spawn_main_zone(commands, zone, region_id, spawner_name, spawner_entity);
}

fn spawn_banned_zone(
    mut commands: Commands,
    banned_zone: &map::Zone,
    region_id: RegionId,
    spawner_name: &Option<String>,
    spawner_entity: Entity,
) {
    let banned_zone_name = match banned_zone.name() {
        Some(name) => Name::new(name.clone()),
        None => Name::new(format!(
            "BannedZone_{}_{}",
            region_id,
            spawner_name.clone().unwrap_or_default()
        )),
    };

    let banned_zone_entity = commands
        .spawn((BannedSpawnZone, banned_zone_name, banned_zone.clone()))
        .id();
    commands
        .entity(banned_zone_entity)
        .insert(DespawnChildOf(spawner_entity));
}

fn spawn_main_zone(
    mut commands: Commands,
    zone: &map::Zone,
    region_id: RegionId,
    spawner_name: &Option<String>,
    spawner_entity: Entity,
) {
    let zone_name = match zone.name() {
        Some(name) => Name::new(name.clone()),
        None => Name::new(format!(
            "Zone_{}_{}",
            region_id,
            spawner_name.clone().unwrap_or_default()
        )),
    };

    let zone_entity = commands.spawn((SpawnZone, zone_name, zone.clone())).id();
    commands
        .entity(zone_entity)
        .insert(DespawnChildOf(spawner_entity));
}

fn spawn_npc_on_spawner(
    par_commands: ParallelCommands,
    world_map_query: WorldMapQuery,
    mut spawners: Query<(
        Entity,
        Mut<Spawner>,
        Ref<Transform>,
        Option<Ref<DespawnChildren>>,
    )>,
    spawn_zones: Query<(Ref<Collider>, Ref<Transform>), With<SpawnZone>>,
    banned_spawn_zones: Query<Ref<Collider>, With<BannedSpawnZone>>,
    time: Res<Time>,
) {
    spawners
        .par_iter_mut()
        .for_each(|(entity, mut spawner, transform, child_zones)| {
            par_commands.command_scope(|commands| {
                // Don't spawn before region loaded
                if world_map_query
                    .region_geodata_from_pos(transform.translation)
                    .is_err()
                {
                    return;
                }
                process_spawner_npcs(
                    commands,
                    &world_map_query,
                    entity,
                    &mut spawner,
                    &child_zones,
                    spawn_zones,
                    banned_spawn_zones,
                    &time,
                );
            });
        });
}

fn process_spawner_npcs(
    mut commands: Commands,
    world_map_query: &WorldMapQuery,
    spawner_entity: Entity,
    spawner: &mut Spawner,
    child_zones: &Option<Ref<DespawnChildren>>,
    spawn_zones: Query<(Ref<Collider>, Ref<Transform>), With<SpawnZone>>,
    banned_spawn_zones: Query<Ref<Collider>, With<BannedSpawnZone>>,
    time: &Time,
) {
    for npc_spawn in spawner.npcs.iter_mut() {
        if npc_spawn.fullfilled() {
            continue;
        }

        if !npc_spawn.timer().finished() {
            npc_spawn.timer_mut().tick(time.delta());
            continue;
        }

        let npc_id = npc_spawn.id();

        // Spawn NPC at fixed location if specified
        if npc_spawn.loc().is_some() {
            spawn_npc_at_location(commands.reborrow(), npc_id, npc_spawn, spawner_entity);
        }

        // Spawn NPCs in zones
        if let Some(child_zones) = child_zones {
            spawn_npcs_in_zones(
                commands.reborrow(),
                world_map_query,
                npc_id,
                npc_spawn,
                spawner_entity,
                child_zones.as_ref(),
                spawn_zones,
                banned_spawn_zones,
            );
        }

        npc_spawn.timer_mut().reset();
    }
}

fn spawn_npc_at_location(
    mut commands: Commands,
    npc_id: npc::Id,
    npc_spawn: &NpcSpawnInfo,
    spawner_entity: Entity,
) {
    commands.trigger_targets(
        npc::Spawn {
            id: npc_id,
            transform: npc_spawn.transform(),
        },
        spawner_entity,
    );
}

fn spawn_npcs_in_zones(
    mut commands: Commands,
    world_map_query: &WorldMapQuery,
    npc_id: npc::Id,
    npc_spawn: &NpcSpawnInfo,
    spawner_entity: Entity,
    child_zones: &DespawnChildren,
    spawn_zones: Query<(Ref<Collider>, Ref<Transform>), With<SpawnZone>>,
    banned_spawn_zones: Query<Ref<Collider>, With<BannedSpawnZone>>,
) {
    for zone_entity in child_zones.iter() {
        if let Ok((zone_collider, zone_transform)) = spawn_zones.get(zone_entity) {
            let Ok(geodata) = world_map_query.region_geodata_from_pos(zone_transform.translation)
            else {
                continue;
            };

            let banned_zones: Vec<_> = child_zones
                .iter()
                .filter_map(|child_entity| banned_spawn_zones.get(child_entity).ok())
                .collect();

            spawn_npcs_in_zone(
                commands.reborrow(),
                npc_id,
                npc_spawn,
                spawner_entity,
                &zone_collider,
                &zone_transform,
                geodata,
                &banned_zones,
            );
        }
    }
}

fn spawn_npcs_in_zone(
    mut commands: Commands,
    npc_id: npc::Id,
    npc_spawn: &NpcSpawnInfo,
    spawner_entity: Entity,
    zone_collider: &Collider,
    zone_transform: &Transform,
    geodata: &RegionGeoData,
    banned_zones: &[Ref<Collider>],
) {
    const MAX_ATTEMPTS: u32 = 10;

    for _ in 0..npc_spawn.needed() {
        let spawn_point = find_valid_spawn_point(
            zone_collider,
            zone_transform,
            geodata,
            banned_zones,
            MAX_ATTEMPTS,
        );

        if let Some(spawn_point) = spawn_point {
            let mut npc_transform = npc_spawn.transform();
            npc_transform.translation = spawn_point;

            commands.trigger_targets(
                npc::Spawn {
                    id: npc_id,
                    transform: npc_transform,
                },
                spawner_entity,
            );
        }
    }
}

fn find_valid_spawn_point(
    zone_collider: &Collider,
    zone_transform: &Transform,
    geodata: &RegionGeoData,
    banned_zones: &[Ref<Collider>],
    max_attempts: u32,
) -> Option<Vec3> {
    let mut spawn_point = zone_collider.generate_random_point_with_geo(zone_transform, geodata);
    let mut attempts = 0;

    while attempts < max_attempts {
        if !is_point_in_banned_zone(spawn_point, banned_zones) {
            return Some(spawn_point);
        }

        spawn_point = zone_collider.generate_random_point_with_geo(zone_transform, geodata);
        attempts += 1;
    }

    None
}

fn is_point_in_banned_zone(point: Vec3, banned_zones: &[Ref<Collider>]) -> bool {
    banned_zones
        .iter()
        .any(|banned_zone| banned_zone.contains_point(Vec3::default(), Rotation::default(), point))
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::chronicles::CHRONICLE;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_spawns_from_json() {
        let curr_dir = std::env::current_dir().expect("Failed to get current directory");
        let path = curr_dir.join("data\\tests\\spawns.json");

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open {:?}", path));
        let reader = BufReader::new(file);

        let spawn_list: Vec<Spawner> = from_reader(reader).expect("Failed to parse NPC from JSON");

        assert_eq!(spawn_list.len(), 198);
    }

    #[test]
    fn test_parse_all_spawns_from_json() {
        let curr_dir = std::env::current_dir().expect("Failed to get current directory");
        let items_dir = curr_dir.join(format!("data\\spawns\\{}", CHRONICLE));
        println!("{:?}", items_dir);

        for entry in std::fs::read_dir(items_dir).expect("Failed to read items directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file =
                    File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
                let reader = BufReader::new(file);

                let items_list: SpawnList = from_reader(reader)
                    .unwrap_or_else(|_| panic!("Failed to parse items from JSON: {:?}", path));

                assert!(!items_list.is_empty());
            }
        }
    }
}
