use bevy::{log, prelude::*};
use config::Config;
use game_core::{
    items::{self, ItemsDataQuery},
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{ActionFail, GameServerPacket, NpcHtmlMessage},
    },
    npc::{self, ChatCommand, DialogTemplater, NpcAction, NpcCommand},
    object_id::{ObjectIdManager, QueryByObjectId},
    stats::NameTitle,
    teleport::{TeleportDestinationTemplate, TeleportDestinations, TeleportDestinationsHandle},
};
use l2r_core::assets::html::TeraHtmlTemplater;
use spatial::FlatDistance;

pub(super) fn handle(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    npcs: Query<(
        Entity,
        Ref<npc::Id>,
        Ref<npc::Kind>,
        Ref<Name>,
        Ref<NameTitle>,
    )>,
    transforms: Query<Ref<Transform>>,
    object_id_manager: Res<ObjectIdManager>,
    dialog_templater: Res<DialogTemplater>,
    config: Res<Config>,

    teleport_dest_handle: Res<TeleportDestinationsHandle>,
    teleport_dest_assets: Res<Assets<TeleportDestinations>>,
    items_data_query: ItemsDataQuery,
) {
    let BypassCommandExecuted(cmd) = trigger.event();

    if let BypassCommand::Npc(NpcAction {
        npc_oid,
        command: NpcCommand::Chat(chat_command),
    }) = cmd
    {
        let entity = trigger.target();

        let Ok((npc_entity, npc_id, npc_kind, npc_name, npc_title)) =
            npcs.by_object_id(*npc_oid, object_id_manager.as_ref())
        else {
            return;
        };

        let Ok(requester_transform) = transforms.get(entity) else {
            return;
        };

        let Ok(target_transform) = transforms.get(npc_entity) else {
            return;
        };

        let distance = requester_transform
            .translation
            .flat_distance(&target_transform.translation);

        if distance > 150.0 {
            log::warn!(
                "NPC: {} is too far away for chat command: {:?}",
                npc_oid,
                chat_command
            );
            return;
        }

        match chat_command {
            ChatCommand::Number(page_number) => {
                let mut context = tera::Context::new();

                context.insert("object_id", npc_oid);
                context.insert("name", npc_name.as_str());
                context.insert("npc_title", npc_title.as_str());

                let page_name = if *page_number == 0 {
                    "index"
                } else {
                    &format!("{page_number}")
                };

                let html = dialog_templater.npc_dialog_with_context(
                    npc_id.as_ref(),
                    npc_kind.as_ref(),
                    page_name,
                    &context,
                );

                match html {
                    Ok(html) => {
                        commands.trigger_targets(
                            GameServerPacket::from(NpcHtmlMessage::new(
                                *npc_oid,
                                html,
                                items::Id::default(),
                            )),
                            entity,
                        );
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to render HTML for NPC with ID: {}: {}",
                            *npc_id,
                            err
                        );
                    }
                }
                commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
            }

            ChatCommand::Tp(tp_list_kind) => {
                // Handle the teleport chat command
                log::info!("NPC: {} - Teleport command executed", npc_oid);

                if let npc::Kind::Teleporter(teleporter_info) = npc_kind.as_ref() {
                    let Some(npc_tps) = teleporter_info
                        .as_ref()
                        .and_then(|info| info.get(tp_list_kind).cloned())
                    else {
                        log::error!("Teleporter info not found for NPC: {}", npc_oid);
                        commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
                        return;
                    };

                    let Some(destinations_table) =
                        teleport_dest_assets.get(teleport_dest_handle.id())
                    else {
                        log::error!("Teleport destinations not found for NPC: {}", npc_oid);
                        commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
                        return;
                    };

                    let destinations = destinations_table.get_many(npc_tps);

                    // Get Item names for each destination
                    let destinations = destinations
                        .into_iter()
                        .filter_map(|(id, dest)| {
                            let Ok(item_info) = items_data_query.get_item_info(dest.item) else {
                                return None;
                            };

                            let mut dest = dest.clone();

                            if config.gameplay().free_teleports {
                                dest.price = 0;
                            }

                            Some(TeleportDestinationTemplate::new(id, dest, item_info.name()))
                        })
                        .collect::<Vec<_>>();

                    let mut context = tera::Context::new();
                    context.insert("object_id", npc_oid);
                    context.insert("name", npc_name.as_str());
                    context.insert("npc_title", npc_title.as_str());
                    context.insert("destinations", &destinations);

                    let path = format!(
                        "{}/{}.html",
                        npc_kind.to_string().to_lowercase().as_str(),
                        "_common/destinations_list"
                    );

                    let html = dialog_templater.render_with_fallback(&path, &context);

                    match html {
                        Ok(html) => {
                            commands.trigger_targets(
                                GameServerPacket::from(NpcHtmlMessage::new(
                                    *npc_oid,
                                    html,
                                    items::Id::default(),
                                )),
                                entity,
                            );
                            commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
                        }
                        Err(err) => {
                            log::error!(
                                "Failed to render HTML for NPC with ID: {}: {}",
                                *npc_id,
                                err
                            );
                        }
                    }
                };
            }
        }
        commands.trigger_targets(GameServerPacket::from(ActionFail), entity);
    }
}
