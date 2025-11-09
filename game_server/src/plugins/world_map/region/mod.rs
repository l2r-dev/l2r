use bevy::prelude::*;
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use game_core::{
    character::Character,
    items::{self, Item, RegionalItemsFolder, SpawnExisting},
    npc::RegionalNpcHandles,
    object_id::ObjectId,
    spawner::{RegionalSpawnersFolder, SpawnListHandle, Spawner},
};
use l2r_core::{
    chronicles::CHRONICLE,
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    plugins::custom_hierarchy::*,
};
use map::{
    LoadRegionItems, NamedZones, Region, RegionComponentsPlugin, RegionGeoData,
    RegionalZonesFolder, WorldMap, ZoneKindVariant, ZoneListHandle,
    id::RegionId,
    info::{RegionInfo, RegionInfoHandle, RegionRespawnZone},
};
use sea_orm::ColumnTrait;
use state::GameServerStateSystems;
use std::path::PathBuf;

pub struct RegionPlugin;
impl Plugin for RegionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RegionComponentsPlugin);

        app.add_observer(load_region_items);

        app.add_systems(
            Update,
            (
                load_region_with_players.in_set(GameServerStateSystems::Run),
                deactivate_empty_region.in_set(GameServerStateSystems::Run),
                loaded_region_geodata,
                loaded_region_info,
                sort_entities_into_folders,
            ),
        );
    }
}

fn load_region_with_players(
    online_players: Query<(&Name, &Transform), (With<Character>, Changed<Transform>)>,
    asset_server: Res<AssetServer>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
) {
    for (name, transform) in online_players.iter() {
        let region_with_player: RegionId = transform.translation.into();
        let region_ids_to_activate = vec![region_with_player];
        // for direction in NavigationDirection::BASIC.iter() {
        //     region_ids_to_activate.push(region_with_player.get_adjacent(*direction));
        // }
        region_ids_to_activate.into_iter().for_each(|region_id| {
            if !world_map.contains_key(&region_id) {
                trace!("Character {:?} activated region {}", name, region_id);

                let mut spawn_list_path = PathBuf::from("spawns");
                spawn_list_path.push(CHRONICLE);
                spawn_list_path.push(region_id.to_string());
                spawn_list_path.set_extension("json");
                let spawner_handle = SpawnListHandle::from(asset_server.load(spawn_list_path));

                // Container for NPC handles in this region, to load/unload assets together with the region
                let npc_handles = RegionalNpcHandles::default();

                let mut geodata_path = PathBuf::from("geo");
                geodata_path.push(region_id.to_string());
                geodata_path.set_extension("l2j");

                let mut zone_list_path = PathBuf::from("zones");
                zone_list_path.push(CHRONICLE);
                zone_list_path.push(region_id.to_string());
                zone_list_path.set_extension("json");

                let zone_list_handle =
                    ZoneListHandle::from(asset_server.load(zone_list_path.clone()));

                let mut region = Region::new(region_id);
                region.set_handle(asset_server.load(geodata_path));
                region.set_zone_list_handle(zone_list_handle);
                let region_name = Name::new(format!("Region-{region_id}"));

                let mut region_info_path = PathBuf::from("regions");
                region_info_path.push(CHRONICLE);
                region_info_path.push(region_id.to_string());
                region_info_path.set_extension("json");
                let region_info_handle =
                    RegionInfoHandle::from(asset_server.load(region_info_path));

                let region_entity = commands
                    .spawn((region_name, spawner_handle, npc_handles, region_info_handle))
                    .id();

                region.set_folder::<ZoneKindVariant>(
                    commands
                        .spawn((RegionalZonesFolder, DespawnChildOf(region_entity)))
                        .id(),
                );
                region.set_folder::<Spawner>(
                    commands
                        .spawn((RegionalSpawnersFolder, DespawnChildOf(region_entity)))
                        .id(),
                );
                region.set_folder::<Item>(
                    commands
                        .spawn((RegionalItemsFolder, DespawnChildOf(region_entity)))
                        .id(),
                );
                commands.entity(region_entity).insert(region);

                world_map.insert(region_id, region_entity);
            }
        });
    }
}

fn sort_entities_into_folders(
    changed_children: Query<(Ref<Region>, Ref<DespawnChildren>), Changed<DespawnChildren>>,
    refs: Query<EntityRef>,
    mut commands: Commands,
) -> Result<()> {
    for (region, despawn_children) in changed_children.iter() {
        for child_entity in despawn_children.iter() {
            let entity_ref = refs.get(child_entity)?;
            commands.insert_into_folders(entity_ref, region.as_ref());
        }
    }
    Ok(())
}

fn loaded_region_geodata(
    mut events: EventReader<AssetEvent<RegionGeoData>>,
    mut regions: Query<(Entity, Mut<Region>)>,
    regions_geodata: Res<Assets<RegionGeoData>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (entity, mut region) in regions.iter_mut() {
                let id = *id;
                if region.handle().id() == id {
                    let region_id = region.id();
                    info!("Geodata for region {} updated", region_id);
                    if let Some(geodata) = regions_geodata.get(id) {
                        region.activate(geodata);
                        commands.trigger_targets(LoadRegionItems, entity);
                    }
                }
            }
        }
    }
}

fn load_region_items(
    load: Trigger<LoadRegionItems>,
    mut commands: Commands,
    regions: Query<Ref<Region>>,
    repo_manager: Res<RepositoryManager>,
) -> Result<()> {
    if repo_manager.is_mock() {
        return Ok(());
    }
    let region_entity = load.target();
    if let Ok(region) = regions.get(region_entity) {
        let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
        let region_id = region.id();

        commands.spawn_task(move || async move {
            let items_models = items_repository
                .find_with_conditions([items::model::Column::OwnerId.is_null()])
                .await?;

            let item_models = items_models
                .into_iter()
                .filter(|item_model| {
                    if let Some(item_coordinates) = item_model.coordinates() {
                        RegionId::from(item_coordinates) == region_id
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();

            AsyncWorld.apply_command(move |world: &mut World| {
                world.trigger_targets(
                    SpawnExisting {
                        item_models,
                        dropped_entity: None,
                        silent: true,
                    },
                    region_entity,
                );
            });

            Ok(())
        });
    }
    Ok(())
}

fn loaded_region_info(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<RegionInfo>>,
    mut regions: Query<(Entity, Ref<RegionInfoHandle>)>,
    regions_info: Res<Assets<RegionInfo>>,
    named_zones: Res<NamedZones>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (region_entity, handle) in regions.iter_mut() {
                let id = *id;
                if handle.id() == id
                    && let Some(info) = regions_info.get(id)
                {
                    let respawn_zone = info.respawn_zone();
                    if let Some(entity) = named_zones.get(respawn_zone).copied() {
                        commands
                            .entity(region_entity)
                            .insert(RegionRespawnZone(entity));
                    }
                }
            }
        }
    }
}

fn deactivate_empty_region(
    online_players: Query<&Transform, With<Character>>,
    mut world_map: ResMut<WorldMap>,
    mut last_time: Local<f32>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if time.elapsed_secs() - *last_time >= 30.0 {
        *last_time = time.elapsed_secs();

        let mut online_player_regions: Vec<RegionId> = online_players
            .iter()
            .map(|transform| transform.translation.into())
            .collect();

        online_player_regions = online_player_regions
            .into_iter()
            .flat_map(|region_id| {
                let adjacent_regions = vec![region_id];
                // for direction in NavigationDirection::BASIC.iter() {
                //     adjacent_regions.push(region_id.get_adjacent(*direction));
                // }
                adjacent_regions
            })
            .collect();

        let active_regions_ids = world_map.active_regions_ids();

        // Get all regions that are in world map but not in online player regions
        let regions_to_deactivate = active_regions_ids
            .into_iter()
            .filter(|region_id| !online_player_regions.contains(region_id))
            .collect::<Vec<RegionId>>();

        regions_to_deactivate.iter().for_each(|&region_id| {
            debug!("Deactivating region {}", region_id);
            if let Some(region_entity) = world_map.get(&region_id).copied() {
                // With the custom hierarchy system, despawning the region will
                // automatically despawn all its children through DespawnChildren hooks
                commands.entity(region_entity).despawn();
                world_map.remove(&region_id);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::{assets::binary::BinaryAsset, utils::get_base_path};
    use spatial::GameVec3;
    use std::{fs::File, io::Read};

    fn read_geodata_from_file(region_id: Option<RegionId>) -> Vec<u8> {
        let path = if let Some(region_id) = region_id {
            let mut path = get_base_path();
            path.push("data");
            path.push("geo");
            path.push(format!("{}.l2j", region_id));
            path
        } else {
            get_base_path().join("data\\tests\\geodata.bin")
        };
        let mut file = File::open(path).expect("Failed to open file");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).expect("Failed to read file");

        bytes
    }

    pub fn create_default_test_region() -> (Region, RegionGeoData) {
        let region_id = RegionId::new(20, 18);
        let geodata = RegionGeoData::from_bytes(&read_geodata_from_file(None)).unwrap();
        let region = Region::new(region_id);
        (region, geodata)
    }

    #[test]
    fn test_region_get_nearest_height() {
        let vec = Vec3::new(28302.0, -4232.0, 11008.0);
        let geo_vec = WorldMap::vec3_to_geo(vec);
        let (_region, geodata) = create_default_test_region();
        let nearest_height = geodata.nearest_height(geo_vec);
        assert_eq!(nearest_height, Some(-4232));
    }

    #[test]
    fn test_calculate_blocks_centers_coordinates() {
        let (region, geodata) = create_default_test_region();
        let centers = region.calculate_blocks_centers_coordinates(&geodata);

        let expected_centers_count = 67538;
        assert_eq!(centers.len(), expected_centers_count);

        assert_eq!(centers[0], GameVec3::new(72, 72, -4672));
        assert_eq!(centers[1], GameVec3::new(72, 200, -4672));
    }

    #[test]
    fn test_can_see_target() {
        let from_location = GameVec3::new(28183, 12440, -3904);
        let to_location = GameVec3::new(28124, 12076, -4088);

        let (_, geodata) = create_default_test_region();
        let can_see = geodata.can_see_target(
            WorldMap::game_to_geo(from_location),
            WorldMap::game_to_geo(to_location),
            1000,
        );
        assert!(can_see);
    }
}
