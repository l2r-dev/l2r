use bevy::prelude::*;
use game_core::{
    network::packets::{
        client::{DoubleSlashCommand, DoubleSlashCommandExecuted},
        server::TeleportToLocation,
    },
    object_id::ObjectId,
    teleport::TeleportType,
};

pub(super) fn handle(
    build_command: Trigger<DoubleSlashCommandExecuted>,
    entities: Query<(&ObjectId, &Transform)>,
    world: &World,
    mut commands: Commands,
) {
    let DoubleSlashCommandExecuted(cmd) = build_command.event();

    if let DoubleSlashCommand::GoTo { target_obj_id } = cmd {
        let Ok(entity_ref) = world.get_entity(build_command.target()) else {
            return;
        };

        let Some(object_id) = entity_ref.get::<ObjectId>() else {
            return;
        };

        for (entity_object_id, transform) in entities.iter() {
            if entity_object_id == target_obj_id {
                commands.trigger_targets(
                    TeleportToLocation::new(*object_id, *transform, TeleportType::default()),
                    build_command.target(),
                );

                break;
            }
        }
    }
}
