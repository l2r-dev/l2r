use avian3d::prelude::{CollisionEventsEnabled, Sensor};
use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use l2r_core::{chronicles::CHRONICLE, plugins::custom_hierarchy::*};
use map::*;
use physics::GameLayer;
use state::{GameServerStateSystems, LoadingSystems};
use std::path::PathBuf;

pub struct ZonesPlugin;
impl Plugin for ZonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ZonesComponentsPlugin);

        app.add_observer(zone_added);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(GlobalZonesFolder);
        });

        app.add_systems(
            Update,
            (
                load_zones::<RespawnPointsZonesList>,
                load_zones::<CastleZonesList>,
                load_zones::<ClanHallZonesList>,
                load_zones::<FortZonesList>,
                load_zones::<OlympiadStadiumZonesList>,
                load_zones::<ResidenceHallTeleportZonesList>,
                load_zones::<ResidenceTeleportZonesList>,
                load_zones::<SiegableHallZonesList>,
                load_zones::<TownZonesList>,
            )
                .in_set(LoadingSystems::AssetInit),
        );

        add_spawn_systems_in_set(app, LoadingSystems::AssetInit);
        add_spawn_systems_in_set(app, GameServerStateSystems::Run);

        app.add_systems(
            Update,
            spawn_regional_zones.in_set(GameServerStateSystems::Run),
        );
    }
}

fn add_spawn_systems_in_set(app: &mut App, set: impl SystemSet) {
    app.add_systems(
        Update,
        (
            spawn_zones::<RespawnPointsZonesList, RespawnPoints>,
            spawn_zones::<CastleZonesList, Castle>,
            spawn_zones::<ClanHallZonesList, ClanHall>,
            spawn_zones::<FortZonesList, Fort>,
            spawn_zones::<OlympiadStadiumZonesList, OlympiadStadium>,
            spawn_zones::<ResidenceHallTeleportZonesList, ResidenceHallTeleport>,
            spawn_zones::<ResidenceTeleportZonesList, ResidenceTeleport>,
            spawn_zones::<SiegableHallZonesList, SiegableHall>,
            spawn_zones::<TownZonesList, Town>,
        )
            .in_set(set),
    );
}

fn zone_added(
    added: Trigger<OnAdd, Zone>,
    mut commands: Commands,
    zones: Query<Ref<Zone>>,
) -> Result<()> {
    let entity = added.target();
    let zone = zones.get(entity)?;

    let center = zone.center();
    let transform = Transform::from_translation(center);
    let collider = zone.collider();

    let zone_bundle = (
        transform,
        ZoneKindVariant::from(zone.kind()),
        collider,
        Sensor,
        GameLayer::player_sensor(),
        CollisionEventsEnabled,
    );

    commands.entity(entity).insert(zone_bundle);
    Ok(())
}

fn load_zones<ZoneListResource>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    zone_list: Option<ResMut<ZoneListResource>>,
) where
    ZoneListResource: Resource + From<ZoneListHandle> + AlwaysLoadedZones,
{
    if zone_list.is_none() {
        let mut zone_list_path = PathBuf::from("zones");
        zone_list_path.push(CHRONICLE);
        zone_list_path.push(format!("_{}", ZoneListResource::name()));
        zone_list_path.set_extension("json");
        let zone_list_handle = ZoneListHandle::from(asset_server.load(zone_list_path));
        commands.insert_resource(ZoneListResource::from(zone_list_handle));
        debug!("Loaded zone list: {:?}", ZoneListResource::name());
    }
}

#[derive(SystemParam)]
pub struct SpawnZoneQuery<'w, 's, ZoneKindComponent: Component> {
    pub commands: Commands<'w, 's>,
    pub zone_lists: Res<'w, Assets<ZoneList>>,
    pub events: EventReader<'w, 's, AssetEvent<ZoneList>>,
    pub existing_zones: Query<'w, 's, Entity, With<ZoneKindComponent>>,
    pub named_zones: ResMut<'w, NamedZones>,
    pub global_zones: Query<'w, 's, Entity, With<GlobalZonesFolder>>,
    pub zone_kind_folders: Query<'w, 's, (Entity, &'static DespawnChildren), With<ZoneKindFolder>>,
}

fn spawn_zones<ZoneListResource, ZoneKindComponent>(
    mut zone_spawn: SpawnZoneQuery<ZoneKindComponent>,
    zone_list_res: Option<Res<ZoneListResource>>,
) -> Result<()>
where
    ZoneListResource: Resource + AsRef<ZoneListHandle> + AlwaysLoadedZones,
    ZoneKindComponent: Component,
{
    if let Some(zone_list_res) = zone_list_res {
        let global_zones_entity = zone_spawn.global_zones.single()?;

        for event in zone_spawn.events.read() {
            let id = zone_list_res.as_ref().as_ref().id();
            if event.is_loaded_with_dependencies(id)
                && let Some(zone_list) = zone_spawn.zone_lists.get(id)
            {
                // Find and despawn folders containing zones of this specific type (ZoneKindComponent).
                // This is needed for hot-reloading: when a zone list asset is modified,
                // we need to remove only the folders that contain zones of the reloaded type,
                // leaving other zone type folders intact.
                for (folder_entity, folder_children) in zone_spawn.zone_kind_folders.iter() {
                    let has_zone_of_kind = folder_children
                        .iter()
                        .any(|child_entity| zone_spawn.existing_zones.contains(child_entity));

                    if has_zone_of_kind {
                        zone_spawn.commands.entity(folder_entity).despawn();
                    }
                }

                // Cache folder entities by zone kind variant.
                // We need this HashMap because entities are spawned via commands and don't exist
                // immediately - they only become available after the command buffer is flushed.
                // The HashMap allows us to reuse the same folder entity for multiple zones
                // of the same kind variant within this iteration.
                let mut kind_folders: HashMap<ZoneKindVariant, Entity> = HashMap::new();

                zone_list.iter().for_each(|zone| {
                    let kind_variant = ZoneKindVariant::from(zone.kind());

                    // Get or create a folder entity for this zone kind variant.
                    // If a folder for this variant was already created in this iteration,
                    // reuse it; otherwise spawn a new folder entity.
                    let zone_kind_entity = *kind_folders.entry(kind_variant).or_insert_with(|| {
                        zone_spawn
                            .commands
                            .spawn((
                                Name::new(kind_variant.to_string()),
                                ZoneKindFolder,
                                DespawnChildOf(global_zones_entity),
                            ))
                            .id()
                    });

                    let zone_name = zone.name();
                    let zone = zone.clone();

                    let entity = zone_spawn
                        .commands
                        .spawn((zone.name_component(), zone))
                        .id();

                    zone_spawn
                        .commands
                        .entity(entity)
                        .insert(DespawnChildOf(zone_kind_entity));

                    if let Some(name) = zone_name {
                        zone_spawn.named_zones.insert(name.clone(), entity);
                    }
                });
            }
        }
    }
    Ok(())
}

#[derive(SystemParam)]
pub struct RegionalSpawnZoneQuery<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub zone_lists: Res<'w, Assets<ZoneList>>,
    pub events: EventReader<'w, 's, AssetEvent<ZoneList>>,
    pub regions: Query<'w, 's, &'static Region>,
    pub despawn_children: Query<'w, 's, &'static DespawnChildren>,
    pub named_zones: Res<'w, NamedZones>,
}

fn spawn_regional_zones(mut regional_spawn: RegionalSpawnZoneQuery) -> Result<()> {
    for event in regional_spawn.events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for region in regional_spawn.regions.iter() {
                let zone_handle = region.zone_list_handle();
                if *id == zone_handle.id()
                    && let Some(zone_list) = regional_spawn.zone_lists.get(*id)
                    && let Some(regional_zones_entity) = region.get_folder::<ZoneKindVariant>()
                {
                    // Despawn all existing regional zone folders and their children.
                    // Regional zones are tied to specific regions, so when a region's zone list
                    // is reloaded, we need to completely rebuild the entire hierarchy.
                    if let Ok(zones_children) =
                        regional_spawn.despawn_children.get(regional_zones_entity)
                    {
                        for child in zones_children.iter() {
                            regional_spawn.commands.entity(child).despawn();
                        }
                    }

                    // Cache folder entities by zone kind variant.
                    // We need this HashMap because entities are spawned via commands and don't exist
                    // immediately - they only become available after the command buffer is flushed.
                    // The HashMap allows us to reuse the same folder entity for multiple zones
                    // of the same kind variant within this iteration.
                    let mut kind_folders: HashMap<ZoneKindVariant, Entity> = HashMap::new();

                    zone_list.iter().for_each(|zone| {
                        let kind_variant = ZoneKindVariant::from(zone.kind());

                        // Get or create a folder entity for this zone kind variant.
                        // If a folder for this variant was already created in this iteration,
                        // reuse it; otherwise spawn a new folder entity.
                        let zone_kind_entity =
                            *kind_folders.entry(kind_variant).or_insert_with(|| {
                                regional_spawn
                                    .commands
                                    .spawn((
                                        Name::new(kind_variant.to_string()),
                                        ZoneKindFolder,
                                        DespawnChildOf(regional_zones_entity),
                                    ))
                                    .id()
                            });

                        let mut zone = zone.clone();

                        if let ZoneKind::Respawn(respawn_zone) = zone.kind_mut()
                            && let Some(entity) =
                                regional_spawn.named_zones.get(respawn_zone.name())
                        {
                            respawn_zone.set_target_entity(*entity);
                        }

                        let zone_entity = regional_spawn
                            .commands
                            .spawn((zone.name_component(), zone))
                            .id();

                        regional_spawn
                            .commands
                            .entity(zone_entity)
                            .insert(DespawnChildOf(zone_kind_entity));
                    });
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::chronicles::CHRONICLE;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_zones_from_json() {
        let curr_dir = std::env::current_dir().expect("Failed to get current directory");
        let path = curr_dir.join("data\\tests\\zones.json");
        println!("{:?}", path);

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file {:?}", path));
        let reader = BufReader::new(file);

        let zone_list: ZoneList = from_reader(reader).expect("Failed to parse items from json");

        println!("{:?}", zone_list.len());
    }

    #[test]
    fn test_parse_all_zones_from_json() {
        let curr_dir = std::env::current_dir().expect("Failed to get current directory");
        let items_dir = curr_dir.join(format!("data\\zones\\{}", CHRONICLE));
        println!("{:?}", items_dir);

        for entry in std::fs::read_dir(items_dir).expect("Failed to read items directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file =
                    File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
                let reader = BufReader::new(file);

                let zones_list: ZoneList = from_reader(reader)
                    .unwrap_or_else(|_| panic!("Failed to parse items from JSON: {:?}", path));

                assert!(!zones_list.is_empty());
            }
        }
    }
}
