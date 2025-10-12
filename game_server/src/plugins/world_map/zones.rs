use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use game_core::custom_hierarchy::{DespawnChildOf, DespawnChildren};
use l2r_core::chronicles::CHRONICLE;
use map::*;
use std::path::PathBuf;

pub struct ZonesPlugin;
impl Plugin for ZonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ZonesComponentsPlugin);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(GlobalZones);
        });

        app.add_systems(
            Update,
            (
                spawn_regional_zones,
                (
                    load_zones::<RespawnPointsZonesList>,
                    spawn_zones::<RespawnPointsZonesList, RespawnPoints>,
                ),
                (
                    load_zones::<CastleZonesList>,
                    spawn_zones::<CastleZonesList, Castle>,
                ),
                (
                    load_zones::<ClanHallZonesList>,
                    spawn_zones::<ClanHallZonesList, ClanHall>,
                ),
                (
                    load_zones::<FortZonesList>,
                    spawn_zones::<FortZonesList, Fort>,
                ),
                (
                    load_zones::<OlympiadStadiumZonesList>,
                    spawn_zones::<OlympiadStadiumZonesList, OlympiadStadium>,
                ),
                (
                    load_zones::<ResidenceHallTeleportZonesList>,
                    spawn_zones::<ResidenceHallTeleportZonesList, ResidenceHallTeleport>,
                ),
                (
                    load_zones::<ResidenceTeleportZonesList>,
                    spawn_zones::<ResidenceTeleportZonesList, ResidenceTeleport>,
                ),
                (
                    load_zones::<SiegableHallZonesList>,
                    spawn_zones::<SiegableHallZonesList, SiegableHall>,
                ),
                (
                    load_zones::<TownZonesList>,
                    spawn_zones::<TownZonesList, Town>,
                ),
            ),
        );

        app.add_systems(Update, load_zones::<ResidenceHallTeleportZonesList>);
        app.add_systems(
            Update,
            spawn_zones::<ResidenceHallTeleportZonesList, ResidenceHallTeleport>,
        );

        app.add_systems(Update, load_zones::<ResidenceTeleportZonesList>);
        app.add_systems(
            Update,
            spawn_zones::<ResidenceTeleportZonesList, ResidenceTeleport>,
        );
    }
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
    }
}

#[derive(SystemParam)]
pub struct SpawnZoneQuery<'w, 's, ZoneKindComponent: Component> {
    pub commands: Commands<'w, 's>,
    pub zone_lists: Res<'w, Assets<ZoneList>>,
    pub events: EventReader<'w, 's, AssetEvent<ZoneList>>,
    pub existing_zones: Query<'w, 's, Entity, With<ZoneKindComponent>>,
    pub named_zones: ResMut<'w, NamedZones>,
    pub global_zones: Query<'w, 's, Entity, With<GlobalZones>>,
    pub zone_kind_containers:
        Query<'w, 's, (Entity, Ref<'static, ZoneKindVariant>), With<ZoneKindFolder>>,
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
            match event {
                AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                    if *id == zone_list_res.as_ref().as_ref().id()
                        && let Some(zone_list) = zone_spawn.zone_lists.get(*id)
                    {
                        let mut kind_containers_cache: HashMap<ZoneKindVariant, Entity> =
                            HashMap::new();
                        for (entity, kind) in zone_spawn.zone_kind_containers.iter() {
                            kind_containers_cache.insert(*kind, entity);
                        }

                        for existing_zone in zone_spawn.existing_zones.iter() {
                            zone_spawn.commands.entity(existing_zone).despawn();
                        }
                        zone_list.iter().for_each(|zone| {
                            let kind_variant = ZoneKindVariant::from(zone.kind());
                            let zone_kind_entity = *kind_containers_cache
                                .entry(kind_variant)
                                .or_insert_with(|| {
                                    zone_spawn
                                        .commands
                                        .spawn((
                                            Name::new(kind_variant.to_string()),
                                            kind_variant,
                                            ZoneKindFolder,
                                            DespawnChildOf(global_zones_entity),
                                        ))
                                        .id()
                                });

                            let zone_name = zone.name();
                            let built_zone_components = zone.build();
                            let zone = zone.clone();
                            let entity = if ZoneListResource::need_collider() {
                                zone_spawn
                                    .commands
                                    .spawn((zone.name_component(), zone, built_zone_components))
                                    .id()
                            } else {
                                zone_spawn
                                    .commands
                                    .spawn((
                                        zone.name_component(),
                                        zone,
                                        built_zone_components.0,
                                        built_zone_components.1,
                                    ))
                                    .id()
                            };

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
                _ => {}
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
    pub region_children: Query<'w, 's, &'static DespawnChildren>,
    pub regional_zones: Query<'w, 's, Entity, With<RegionalZones>>,
    pub zone_kind_containers:
        Query<'w, 's, (Entity, Ref<'static, ZoneKindVariant>), With<ZoneKindFolder>>,
    pub zones: Query<'w, 's, Entity, With<Zone>>,
    pub named_zones: Res<'w, NamedZones>,
}

fn spawn_regional_zones(mut regional_spawn: RegionalSpawnZoneQuery) -> Result<()> {
    for event in regional_spawn.events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                let mut kind_containers_cache: HashMap<ZoneKindVariant, Entity> = HashMap::new();
                for (entity, kind) in regional_spawn.zone_kind_containers.iter() {
                    kind_containers_cache.insert(*kind, entity);
                }

                for region in regional_spawn.regions.iter() {
                    let zone_handle = region.zone_list_handle();
                    if *id == zone_handle.id()
                        && let Some(zone_list) = regional_spawn.zone_lists.get(*id)
                    {
                        let regional_zones_entity = regional_spawn.regional_zones.single()?;

                        if let Ok(zones_children) =
                            regional_spawn.region_children.get(regional_zones_entity)
                        {
                            for child in zones_children.iter() {
                                if regional_spawn.zones.get(child).is_ok() {
                                    regional_spawn.commands.entity(child).despawn();
                                }
                            }
                        }

                        zone_list.iter().for_each(|zone| {
                            let kind_variant = ZoneKindVariant::from(zone.kind());
                            let zone_kind_entity = *kind_containers_cache
                                .entry(kind_variant)
                                .or_insert_with(|| {
                                    regional_spawn
                                        .commands
                                        .spawn((
                                            Name::new(kind_variant.to_string()),
                                            kind_variant,
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
                            let built_zone_components = zone.build();
                            let zone_entity = regional_spawn
                                .commands
                                .spawn((zone.name_component(), zone, built_zone_components))
                                .id();
                            regional_spawn
                                .commands
                                .entity(zone_entity)
                                .insert(DespawnChildOf(zone_kind_entity));
                        });
                    }
                }
            }
            _ => {}
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
