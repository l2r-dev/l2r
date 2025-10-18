use bevy::{log, prelude::*};
use game_core::{
    network::packets::client::{DoubleSlashCommand, DoubleSlashCommandExecuted},
    npc,
};

pub(super) fn handle(
    build_command: Trigger<DoubleSlashCommandExecuted>,
    mut commands: Commands,
    transforms: Query<Ref<Transform>>,
) {
    let DoubleSlashCommandExecuted(cmd) = build_command.event();

    let entity = build_command.target();

    if let DoubleSlashCommand::Spawn(npc_id) = cmd {
        let npc_id = *npc_id;

        let current_transform = match transforms.get(entity) {
            Ok(transform) => transform,
            Err(_) => {
                log::error!("Failed to get transform for entity: {:?}", entity);
                return;
            }
        };

        commands.trigger_targets(
            npc::Spawn {
                id: npc_id,
                transform: *current_transform,
            },
            entity,
        );
    }
}
