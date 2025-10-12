use bevy::{log, prelude::*};
use game_core::{
    character::Character,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{ActionFail, GameServerPacket, TeleportToLocation},
    },
    npc::{self, NpcAction, NpcCommand},
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId},
    teleport::{TeleportDestinations, TeleportDestinationsHandle, TeleportType},
};
use spatial::FlatDistance;

const TELEPORT_ACTIVATION_DISTANCE: f32 = 150.0;

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    object_id_manager: Res<ObjectIdManager>,

    characters: Query<(Ref<ObjectId>, Ref<Transform>), With<Character>>,
    teleport_dest_handle: Res<TeleportDestinationsHandle>,
    teleport_dest_assets: Res<Assets<TeleportDestinations>>,
    npcs: Query<(Ref<Name>, Ref<npc::Kind>, Ref<Transform>)>,
) {
    let BypassCommandExecuted(cmd) = trigger.event();

    if let BypassCommand::Npc(NpcAction {
        npc_oid,
        command: NpcCommand::Tp(tp_id),
    }) = cmd
    {
        let entity = trigger.target();

        // TODO: Save previous location before teleporting

        log::debug!("NPC: {}, Teleport id: {}", npc_oid, tp_id);

        let Some(destinations_table) = teleport_dest_assets.get(teleport_dest_handle.id()) else {
            log::error!("Teleport destinations not found for NPC: {}", npc_oid);
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return;
        };

        let Some(destination) = destinations_table.get(tp_id) else {
            log::error!("Teleport destination not found: {}", tp_id);
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return;
        };

        let Ok((object_id, transform)) = characters.get(entity) else {
            log::error!("Failed to get character details for entity {:?}", entity);
            return;
        };

        let Ok((npc_name, npc_kind, npc_transform)) =
            npcs.by_object_id(*npc_oid, object_id_manager.as_ref())
        else {
            log::error!("Failed to get NPC kind for entity {:?}", entity);
            return;
        };

        // Check if the NPC is close enough to the character
        if npc_transform
            .translation
            .flat_distance(&transform.translation)
            > TELEPORT_ACTIVATION_DISTANCE
        {
            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            return;
        }

        // TODO: Some TP may be restricted to level, or other conditions?

        match npc_kind.as_ref() {
            npc::Kind::Teleporter(Some(teleporter_info)) => {
                if !teleporter_info.has_id(tp_id) {
                    log::error!(
                        "NPC {} {} does not have destination id {}",
                        npc_oid,
                        npc_name.as_str(),
                        tp_id
                    );
                    commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
                    return;
                }
            }
            _ => {
                log::error!(
                    "NPC {:?} {} is not a teleporter",
                    npc_kind,
                    npc_name.as_str()
                );
                commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
                return;
            }
        }

        commands.trigger_targets(
            TeleportToLocation::new(
                *object_id,
                Transform::from_translation(destination.location.into()),
                TeleportType::default(),
            ),
            entity,
        );
    }
}
