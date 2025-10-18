use bevy::{log, prelude::*};
use bevy_asset::LoadedFolder;
use game_core::{
    movement::MoveToEntity,
    network::packets::{
        client::{BypassCommand, BypassCommandExecuted},
        server::{ActionFail, GameServerPacket, NpcHtmlMessage},
    },
    npc::*,
    object_id::ObjectId,
    path_finding::{InActionPathfindingTimer, VisibilityCheckRequest},
    stats::*,
};
use l2r_core::{
    assets::html::{HtmlAsset, TeraHtmlTemplater},
    chronicles::CHRONICLE,
};
use map::{WorldMap, WorldMapQuery};
use spatial::{FlatDistance, GameVec3};
use std::path::PathBuf;

const DIALOG_RANGE: f32 = 100.0;

pub struct DialogPlugin;
impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DialogComponentsPlugin);

        app.add_systems(Startup, load_npc_dialog_assets)
            .add_systems(Update, (html_assets_changed, templates_folder_changed))
            .add_systems(Update, dialog_request_handler);

        app.add_observer(send_npc_info_dialog);
    }
}

fn dialog_request_handler(
    mut commands: Commands,
    world_map_query: WorldMapQuery,
    requests: Query<(Entity, Ref<DialogRequest>), Without<InActionPathfindingTimer>>,
    move_to: Query<Ref<MoveToEntity>>,
    transforms: Query<Ref<Transform>>,
    npcs: Query<Ref<ObjectId>, With<Kind>>,
) -> Result<()> {
    for (requester, request) in &mut requests.iter() {
        let dialog_target = **request;

        let Ok(requester_transform) = transforms.get(requester) else {
            continue;
        };

        let Ok(target_transform) = transforms.get(dialog_target) else {
            continue;
        };

        let requester_pos = requester_transform.translation;
        let target_pos = target_transform.translation;
        let distance = requester_pos.flat_distance(&target_pos);

        // Target is out of range, need to move closer
        if distance > DIALOG_RANGE {
            // Check if already moving to the correct target
            if let Ok(move_to) = move_to.get(requester)
                && move_to.target == dialog_target
            {
                continue;
            }

            let geodata = world_map_query.region_geodata_from_pos(requester_pos)?;

            // Use the same logic as follow/attack plugins - check line of sight
            let can_move_to = geodata.can_move_to(
                &WorldMap::vec3_to_geo(requester_pos),
                &WorldMap::vec3_to_geo(target_pos),
            );

            if can_move_to {
                // Direct line of sight, use simple movement
                commands.entity(requester).try_insert(MoveToEntity {
                    target: dialog_target,
                    range: DIALOG_RANGE,
                });
            } else {
                // No line of sight, use pathfinding via visibility check
                commands
                    .entity(requester)
                    .try_insert(InActionPathfindingTimer::default());

                commands.trigger_targets(
                    VisibilityCheckRequest {
                        entity: requester,
                        start: requester_pos,
                        target: target_pos,
                    },
                    requester,
                );
            }
        } else {
            // Within range, check line of sight and open dialog
            let geodata = world_map_query.region_geodata_from_pos(requester_pos)?;

            if !geodata.can_see_target(
                WorldMap::vec3_to_geo(requester_pos),
                WorldMap::vec3_to_geo(target_pos),
            ) {
                commands
                    .entity(requester)
                    .remove::<(MoveToEntity, DialogRequest)>();
                commands.trigger_targets(GameServerPacket::from(ActionFail), requester);
                continue;
            }

            commands
                .entity(requester)
                .remove::<(MoveToEntity, DialogRequest)>();

            if let Ok(object_id) = npcs.get(dialog_target) {
                commands.trigger_targets(
                    BypassCommandExecuted::from(BypassCommand::Npc(NpcAction {
                        npc_oid: *object_id,
                        // Show index page
                        command: NpcCommand::Chat(ChatCommand::Number(0)),
                    })),
                    requester,
                );
            }
        }
    }
    Ok(())
}

fn load_npc_dialog_assets(
    asset_server: Res<AssetServer>,
    mut npc_dialog_handles: ResMut<NpcDialogHandles>,
) {
    let mut asset_dir = PathBuf::new();
    asset_dir.push("html");
    asset_dir.push(CHRONICLE);
    asset_dir.push("npc");

    let loaded_folder = asset_server.load_folder(asset_dir);
    npc_dialog_handles.folder = loaded_folder.clone();
}

fn templates_folder_changed(
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    asset_folders: Res<Assets<LoadedFolder>>,
    mut npc_dialog_handles: ResMut<NpcDialogHandles>,
    mut npc_dialog: ResMut<DialogTemplater>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                if npc_dialog_handles.folder.id() == *id
                    && let Some(loaded_folder) = asset_folders.get(npc_dialog_handles.folder.id())
                {
                    npc_dialog_handles.htmls = Vec::with_capacity(loaded_folder.handles.len());

                    for handle in loaded_folder.handles.iter() {
                        let typed_handle = handle.clone().typed_unchecked::<HtmlAsset>();

                        npc_dialog_handles.htmls.push(typed_handle);
                    }
                    npc_dialog.reload()
                }
            }
            _ => {}
        }
    }
}

fn html_assets_changed(
    mut events: EventReader<AssetEvent<HtmlAsset>>,
    npc_dialog_handles: Res<NpcDialogHandles>,
    mut npc_dialog: ResMut<DialogTemplater>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && npc_dialog_handles.htmls.iter().any(|h| h.id() == *id)
        {
            npc_dialog.reload()
        }
    }
}

fn send_npc_info_dialog(
    trigger: Trigger<SendNpcInfoDialog>,
    dialog_templater: Res<DialogTemplater>,
    mut commands: Commands,
    npcs: Query<NpcQuery>,
    npc_info: RegionalNpcInfoQuery,
) -> Result<()> {
    let character_entity = trigger.target();
    let npc_entity = trigger.event().0;

    let npc = npcs.get(npc_entity)?;
    let npc_model = npc_info.get(npc_entity)?;

    let mut context = tera::Context::new();
    context.insert("object_id", npc.object_id);
    context.insert("npc_id", npc.id);
    context.insert("name", &npc.name.as_str());
    context.insert("kind", &npc.kind.to_string());

    if let Some(title) = npc.title {
        context.insert("npc_title", &title.as_str());
    }

    let game_position = GameVec3::from(npc.transform.translation);

    // Position information
    context.insert(
        "position",
        &format!(
            "X: {:.0}, Y: {:.0}, Z: {:.0}",
            game_position.x, game_position.y, game_position.z
        ),
    );

    // Status flags
    context.insert("is_attackable", &npc.attackable.is_some());
    context.insert("is_in_combat", &npc.in_combat.is_some());
    context.insert("is_dead", &npc.dead.is_some());

    // Vitals information
    context.insert("hp", &npc.condition.get(VitalsStat::Hp));
    context.insert("max_hp", &npc.condition.get(VitalsStat::MaxHp));
    context.insert("mp", &npc.condition.get(VitalsStat::Mp));
    context.insert("max_mp", &npc.condition.get(VitalsStat::MaxMp));
    context.insert("cp", &npc.condition.get(VitalsStat::Cp));
    context.insert("max_cp", &npc.condition.get(VitalsStat::MaxCp));

    // Attack stats
    context.insert("p_atk", &npc.attack_stats.get(AttackStat::PAtk));
    context.insert("m_atk", &npc.attack_stats.get(AttackStat::MAtk));
    context.insert("atk_spd", &npc.attack_stats.get(AttackStat::PAtkSpd));
    context.insert("cast_spd", &npc.attack_stats.get(AttackStat::CastSpd));

    // Defence stats
    context.insert("p_def", &npc.defence_stats.get(DefenceStat::PDef));
    context.insert("m_def", &npc.defence_stats.get(DefenceStat::MDef));

    // Critical stats
    context.insert(
        "crit_rate",
        &npc.critical_stats.get(CriticalStat::CriticalRate),
    );
    context.insert(
        "crit_damage",
        &npc.critical_stats.get(CriticalStat::CriticalDamage),
    );

    // Progress rewards
    context.insert("exp_reward", &npc.progress_reward.exp);
    context.insert("sp_reward", &npc.progress_reward.sp);

    // Drop table information
    if let Some(drop_table) = &npc_model.drop_table {
        let calculated_drops = drop_table.calculate_drops();
        let calculated_spoils = drop_table.calculate_spoils();

        let drop_items = drop_table.get_drop_items_json();
        let spoil_items = drop_table.get_spoil_items_json();

        context.insert("drop_items", &drop_items);
        context.insert("spoil_items", &spoil_items);
        context.insert("calculated_drops", &calculated_drops);
        context.insert("calculated_spoils", &calculated_spoils);
    }

    // NPC model data
    if let Some(display_id) = &npc_model.display_id {
        context.insert("display_id", display_id);
    }
    context.insert("level", &npc_model.level);
    context.insert("race", &npc_model.race);
    context.insert("gender", &npc_model.gender);

    let path = "_common/npc_info.html";
    let html = dialog_templater.render_with_fallback(path, &context);

    match html {
        Ok(html) => {
            commands.trigger_targets(
                GameServerPacket::from(NpcHtmlMessage::new(
                    *npc.object_id,
                    html,
                    game_core::items::Id::from(0), // Default item ID
                )),
                character_entity,
            );
            commands.trigger_targets(GameServerPacket::from(ActionFail), character_entity);
        }
        Err(err) => {
            log::error!("Failed to render HTML for NPC with ID: {}: {}", npc.id, err);
            commands.trigger_targets(GameServerPacket::from(ActionFail), character_entity);
        }
    }
    Ok(())
}
