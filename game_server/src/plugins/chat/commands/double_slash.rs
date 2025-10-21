use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    admin_menu::AdminMenuCommand,
    character, items,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{BypassCommand, BypassCommandExecuted, DoubleSlashCommand, GameClientPacket},
            server::TeleportToLocation,
        },
        session::GetCharEntity,
    },
    npc,
    object_id::ObjectId,
    teleport::TeleportType,
};
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
    named_entities: Query<(&ObjectId, &Transform, &Name)>,
    world: &World,
    mut commands: Commands,
) -> Result {
    let event = receive.event();

    let initiator_entity =
        sessions.get_character_entity(&event.connection.id(), &character_tables)?;
    let initiator_entity_ref = world.get_entity(initiator_entity)?;
    let initiator_object_id = initiator_entity_ref
        .get::<ObjectId>()
        .ok_or("no object_id on entity")?;

    let GameClientPacket::DoubleSlashCommand(ref packet) = event.packet else {
        return Ok(());
    };

    match packet {
        DoubleSlashCommand::Unknown => {}

        DoubleSlashCommand::Admin => {
            commands.trigger_targets(
                BypassCommandExecuted(BypassCommand::Admin(AdminMenuCommand::Main)),
                initiator_entity,
            );
        }
        DoubleSlashCommand::Spawn { npc_id } => {
            commands.trigger_targets(
                npc::Spawn {
                    id: *npc_id,
                    transform: *entities.get(initiator_entity)?.1,
                },
                initiator_entity,
            );
        }

        DoubleSlashCommand::GoTo { target_obj_id } => {
            if let Some((_, transform)) = entities
                .iter()
                .find(|(candidate_object_id, _)| *candidate_object_id == target_obj_id)
            {
                commands.trigger_targets(
                    TeleportToLocation::new(
                        *initiator_object_id,
                        *transform,
                        TeleportType::default(),
                    ),
                    initiator_entity,
                );
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

        DoubleSlashCommand::TeleportTo { target_name } => {
            if let Some((_, transform, _)) = named_entities
                .iter()
                .find(|(_, _, candidate_name)| candidate_name.as_str() == target_name)
            {
                commands.trigger_targets(
                    TeleportToLocation::new(
                        *initiator_object_id,
                        *transform,
                        TeleportType::default(),
                    ),
                    initiator_entity,
                );
            }
        }

        DoubleSlashCommand::Summon { id, count } => {
            const NPC_ID_OFFSET: u32 = 1_000_000;

            if *id < NPC_ID_OFFSET {
                commands.send_event(items::SpawnNew {
                    item_ids: vec![(*id).into()],
                    count: *count,
                    item_location: items::ItemLocation::Inventory,
                    dropped_entity: None,
                    owner: Some(initiator_entity),
                    silent: false, // Show system messages for admin-spawned items
                });
            } else {
                commands.trigger_targets(
                    npc::Spawn {
                        id: (*id - NPC_ID_OFFSET).into(),
                        transform: *entities.get(initiator_entity)?.1,
                    },
                    initiator_entity,
                );
            }
        }

        DoubleSlashCommand::InstantMove => {
            //TODO: нужно выставлять на чара флаг, чтобы при следующем его запросе на передвижение он мгновенно телепортировался
        }

        DoubleSlashCommand::Teleport { x, z } => {
            commands.trigger_targets(
                TeleportToLocation::new(
                    *initiator_object_id,
                    Transform::from_xyz(*x, 0.0, *z),
                    TeleportType::default(),
                ),
                initiator_entity,
            );
        }
    }

    Ok(())
}
