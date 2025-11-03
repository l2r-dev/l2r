use avian3d::{prelude::*, spatial_query::SpatialQuery};
use bevy::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use game_core::{
    character::Character,
    collision_layers::Layer,
    encounters::{EnteredWorld, KnownEntities},
    network::{
        broadcast::{BroadcastScope, ServerPacketBroadcast, ServerPacketsBroadcast},
        session::GameServerSession,
    },
};
use map::id::RegionId;

pub struct NetworkBroadcastPlugin;
impl Plugin for NetworkBroadcastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerPacketBroadcast>()
            .add_event::<ServerPacketsBroadcast>()
            .add_observer(multiple_server_packets_broadcast)
            .add_observer(server_packet_broadcast);
    }
}

fn multiple_server_packets_broadcast(
    broadcast: Trigger<ServerPacketsBroadcast>,
    mut commands: Commands,
) {
    let broadcaster = broadcast.target();
    for packet in broadcast.event().packets.iter() {
        commands.trigger_targets(
            ServerPacketBroadcast {
                packet: packet.clone(),
                scope: broadcast.event().scope.clone(),
            },
            broadcaster,
        );
    }
}

fn server_packet_broadcast(
    broadcast: Trigger<ServerPacketBroadcast>,
    sessions: Query<Entity, With<GameServerSession>>,
    characters: Query<(Entity, &Transform), (With<Character>, With<EnteredWorld>)>,
    broadcasters: Query<&Transform>,
    known_entities: Query<(Entity, Ref<KnownEntities>)>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    let event = broadcast.event();
    let broadcaster = broadcast.target();

    match &event.scope {
        BroadcastScope::All => {
            for session_entity in sessions.iter() {
                commands.trigger_targets(event.packet.clone(), session_entity);
            }
        }
        BroadcastScope::Known => {
            let mut broadcast_targets = EntityHashSet::new();
            // Entities that are known to the broadcaster
            if let Ok((_, known)) = known_entities.get(broadcaster) {
                for entity in known.iter() {
                    if characters.contains(*entity) {
                        broadcast_targets.insert(*entity);
                    }
                }
            }
            // Entities that know the broadcaster
            for (entity, known) in known_entities.iter() {
                if known.contains(&broadcaster) && characters.contains(entity) {
                    broadcast_targets.insert(entity);
                }
            }
            // Exclude the broadcaster
            broadcast_targets.remove(&broadcaster);

            for entity in broadcast_targets.iter() {
                commands.trigger_targets(event.packet.clone(), *entity);
            }
        }
        BroadcastScope::KnownAndSelf => {
            let mut broadcast_targets = EntityHashSet::new();
            // Entities that are known to the broadcaster
            if let Ok((_, known)) = known_entities.get(broadcaster) {
                for entity in known.iter() {
                    if characters.contains(*entity) {
                        broadcast_targets.insert(*entity);
                    }
                }
            }
            // Entities that know the broadcaster
            for (entity, known) in known_entities.iter() {
                if known.contains(&broadcaster) && characters.contains(entity) {
                    broadcast_targets.insert(entity);
                }
            }
            // Include the broadcaster
            broadcast_targets.insert(broadcaster);

            for entity in broadcast_targets.iter() {
                commands.trigger_targets(event.packet.clone(), *entity);
            }
        }
        BroadcastScope::Radius(radius) => {
            if let Ok(broadcaster_transform) = broadcasters.get(broadcaster) {
                let query_sphere = Collider::sphere(*radius);

                let filter = SpatialQueryFilter::default().with_mask(Layer::broadcast_mask());

                let nearby_entities = spatial_query.shape_intersections(
                    &query_sphere,
                    broadcaster_transform.translation,
                    Quat::IDENTITY,
                    &filter,
                );

                for entity in nearby_entities {
                    if characters.contains(entity) {
                        commands.trigger_targets(event.packet.clone(), entity);
                    }
                }
            }
        }
        BroadcastScope::InRegion => {
            if let Ok(broadcaster_transform) = broadcasters.get(broadcaster) {
                let broadcaster_region = RegionId::from(broadcaster_transform.translation);
                for (session_entity, session_transform) in characters.iter() {
                    let session_region = RegionId::from(session_transform.translation);
                    if session_region == broadcaster_region {
                        commands.trigger_targets(event.packet.clone(), session_entity);
                    }
                }
            }
        }
        BroadcastScope::Entities(entities) => {
            for &entity in entities {
                if characters.contains(entity) || sessions.contains(entity) {
                    commands.trigger_targets(event.packet.clone(), entity);
                }
            }
        }
    }
}
