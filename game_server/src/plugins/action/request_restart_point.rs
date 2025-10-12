use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    character::Character,
    custom_hierarchy::DespawnChildOf,
    network::{
        config::GameServerNetworkConfig,
        packets::{client::GameClientPacket, server::TeleportToLocation},
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
    stats::Resurrect,
    teleport::TeleportType,
};
use map::{
    Respawn as RespawnZone, RespawnPoints, SpawnPoint, SpawnPointsGetter, Town, WorldMap, Zone,
    ZoneKind, id::RegionId, info::RegionRespawnZone,
};

pub(crate) struct RequestRestartPointPlugin;

impl Plugin for RequestRestartPointPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    regions: Query<Ref<RegionRespawnZone>>,
    world_map: Res<WorldMap>,
    respawn_zones: Query<(Ref<Zone>, Ref<Collider>, Ref<DespawnChildOf>), With<RespawnZone>>,
    spawn_points_zones: Query<Ref<Zone>, Or<(With<RespawnPoints>, With<Town>)>>,
    characters: Query<(Ref<ObjectId>, Ref<Transform>), With<Character>>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestRestartPoint(ref _packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        let (object_id, char_transform) = characters.get(character_entity)?;

        let region_id = RegionId::from(char_transform.translation);
        if let Some(region_entity) = world_map.get(&region_id).copied() {
            let needed_respawn_zone_entity = regions.get(region_entity).map(|region_spawn_zone| {
                let mut target_entity = None;
                respawn_zones.iter().for_each(|(zone, collider, child_of)| {
                    if child_of.get() == region_entity
                        && let ZoneKind::Respawn(respawn_zone) = zone.kind()
                        && collider.contains_point(
                            Vec3::default(),
                            Quat::default(),
                            char_transform.translation,
                        )
                    {
                        target_entity = respawn_zone.target_entity();
                    }
                });
                target_entity.unwrap_or(region_spawn_zone.0)
            });

            let respawn_point = match needed_respawn_zone_entity {
                Ok(zone_entity) => {
                    let zone = spawn_points_zones.get(zone_entity).unwrap();
                    match zone.kind() {
                        ZoneKind::RespawnPoints(points_zone) => points_zone.spawn_points().random(),
                        ZoneKind::Town(town) => town.spawn_points().random(),
                        _ => SpawnPoint::default(),
                    }
                }
                Err(_) => SpawnPoint::default(),
            };

            commands.trigger_targets(Resurrect, character_entity);

            commands.trigger_targets(
                TeleportToLocation::new(
                    *object_id,
                    Transform::from_translation(respawn_point.into()),
                    TeleportType::default(),
                ),
                character_entity,
            );
        }
    }
    Ok(())
}
