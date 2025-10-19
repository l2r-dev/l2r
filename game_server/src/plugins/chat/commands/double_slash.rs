use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{admin_menu::AdminMenuCommand, character, items, network::{
    config::GameServerNetworkConfig,
    packets::{
        client::{BypassCommand, BypassCommandExecuted, DoubleSlashCommand, GameClientPacket},
        server::TeleportToLocation,
    },
    session::GetCharEntity,
}, npc, object_id::ObjectId, teleport::TeleportType};
use l2r_core::model::session::ServerSessions;

pub struct BuildCommandsPlugin;
impl Plugin for BuildCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_packet);
    }
}

fn handle_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    sessions: Res<ServerSessions>,
    character_tables: Query<Ref<character::Table>>,
    entities: Query<(&ObjectId, &Transform)>,
    world: &World,
    transforms: Query<Ref<Transform>>,
    mut commands: Commands,
) {
    let event = receive.event();

    if let GameClientPacket::DoubleSlashCommand(ref packet) = event.packet
        && let Ok(initiator_entity) =
            sessions.get_character_entity(&event.connection.id(), &character_tables)
    {
        match packet {
            DoubleSlashCommand::Unknown => {}

            DoubleSlashCommand::Admin => {
                commands.trigger_targets(
                    BypassCommandExecuted(BypassCommand::Admin(AdminMenuCommand::Main)),
                    initiator_entity,
                );
            }
            DoubleSlashCommand::Spawn { npc_id } => {
                let npc_id = *npc_id;

                let current_transform = match transforms.get(initiator_entity) {
                    Ok(transform) => transform,
                    Err(_) => {
                        log::error!("Failed to get transform for entity: {:?}", initiator_entity);
                        return;
                    }
                };

                commands.trigger_targets(
                    npc::Spawn {
                        id: npc_id,
                        transform: *current_transform,
                    },
                    initiator_entity,
                );
            }

            DoubleSlashCommand::GoTo { target_obj_id } => {
                let Ok(entity_ref) = world.get_entity(initiator_entity) else {
                    return;
                };

                let Some(object_id) = entity_ref.get::<ObjectId>() else {
                    return;
                };

                for (entity_object_id, transform) in entities.iter() {
                    if entity_object_id == target_obj_id {
                        commands.trigger_targets(
                            TeleportToLocation::new(
                                *object_id,
                                *transform,
                                TeleportType::default(),
                            ),
                            initiator_entity,
                        );

                        break;
                    }
                }
            }

            DoubleSlashCommand::Item { id, count } => {
                commands.send_event(items::SpawnNew {
                    item_ids: vec![*id],
                    count: *count,
                    item_location: items::ItemLocation::Inventory,
                    dropped_entity: None,
                    owner: Some(initiator_entity),
                    silent: false, // Show system messages for admin-spawned items
                });
            }
        }
    }
}
