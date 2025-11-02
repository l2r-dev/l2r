use crate::plugins::doors::DoorQuery;
use bevy::{platform::collections::HashMap, prelude::*};
use config::Config;
use game_core::{
    action::wait_kind::WaitKind,
    attack::{AttackingList, Dead, DeadTimer},
    character::Character,
    encounters::EnteredWorld,
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{DoorStatusUpdate, GameServerPacket, Revive, UserInfoUpdated},
    },
    object_id::ObjectId,
    stats::*,
};
use l2r_core::chronicles::CHRONICLE;
use state::{LoadingSystems, StatKindSystems};
use std::path::PathBuf;
use strum::IntoEnumIterator;

pub struct VitalsStatsPlugin;
impl Plugin for VitalsStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VitalsStatsComponentsPlugin);

        app.add_systems(
            Update,
            (
                load_assets.in_set(LoadingSystems::AssetInit),
                update_stats_table.in_set(LoadingSystems::AssetInit),
            ),
        );

        app.add_observer(resurrect_handle)
            .add_observer(full_restore_trigger_handle);

        app.add_systems(
            Update,
            on_required_components_changed
                .in_set(StatKindSystems::Vitals)
                .before(stats_changed),
        )
        .add_systems(
            Update,
            (doors_stats_changed, stats_changed)
                .chain()
                .in_set(StatKindSystems::Vitals),
        )
        .add_systems(
            Update,
            full_restore_event_handle
                .in_set(StatKindSystems::Vitals)
                .after(stats_changed),
        )
        .add_systems(
            Update,
            vitals_regeneration
                .in_set(StatKindSystems::Vitals)
                .after(full_restore_event_handle),
        );
    }
}

const VITALS_REGEN_PERIOD: f32 = 3.0;

fn vitals_regeneration(
    mut vitals_stats: Query<(Mut<VitalsStats>, Ref<WaitKind>, Ref<Movable>), Without<Dead>>,
    time: Res<Time>,
    config: Res<Config>,
    mut last_time: Local<f32>,
) {
    let time_spent = time.elapsed_secs() - *last_time;
    if time_spent >= VITALS_REGEN_PERIOD {
        *last_time = time.elapsed_secs();

        vitals_stats
            .par_iter_mut()
            .for_each(|(mut stats, wait_kind, movable)| {
                let vitals_multiplier = if *wait_kind == WaitKind::Sit {
                    1.5
                } else if movable.is_running() && movable.is_moving() {
                    0.7
                } else if !movable.is_moving() {
                    1.1
                } else {
                    1.0
                };

                let vitals_multiplier = vitals_multiplier * config.gameplay().regen_rate;

                let hp_regen = stats.get(VitalsStat::HpRegen) * vitals_multiplier;
                let mp_regen = stats.get(VitalsStat::MpRegen) * vitals_multiplier;
                let cp_regen = stats.get(VitalsStat::CpRegen) * vitals_multiplier;

                let current_hp = stats.get(VitalsStat::Hp);
                let current_mp = stats.get(VitalsStat::Mp);
                let current_cp = stats.get(VitalsStat::Cp);

                let max_hp = stats.get(VitalsStat::MaxHp);
                let max_mp = stats.get(VitalsStat::MaxMp);
                let max_cp = stats.get(VitalsStat::MaxCp);

                if current_hp < max_hp {
                    let new_hp = (current_hp + hp_regen).min(max_hp);
                    stats.insert(VitalsStat::Hp, new_hp);
                }

                if current_mp < max_mp {
                    let new_mp = (current_mp + mp_regen).min(max_mp);
                    stats.insert(VitalsStat::Mp, new_mp);
                }

                if current_cp < max_cp {
                    let new_cp = (current_cp + cp_regen).min(max_cp);
                    stats.insert(VitalsStat::Cp, new_cp);
                }
            });
    }
}

fn stats_changed(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Ref<ObjectId>,
            Mut<VitalsStats>,
            Has<Character>,
            Has<EnteredWorld>,
        ),
        Changed<VitalsStats>,
    >,
    attackers_lists: Query<Ref<AttackingList>>,
) {
    for (entity, object_id, mut vitals_stats, character, entered_world) in query.iter_mut() {
        if character && !entered_world {
            continue;
        }
        for variant in vitals_stats.changed_stats() {
            if *variant == VitalsStat::Hp && vitals_stats.dead() {
                let killer_entity = match attackers_lists.get(entity) {
                    Ok(list) => list.last_attacker().unwrap_or(entity),
                    Err(_) => entity, // No attackers list means self-damage or environmental damage
                };
                commands.trigger_targets(Dead::new(killer_entity), entity);
            }
        }

        let status_update = vitals_stats.diff_status_update(*object_id);
        if let Some(status_update) = status_update {
            commands.trigger_targets(
                ServerPacketBroadcast::new(GameServerPacket::from(status_update)),
                entity,
            );
        }
    }
}

fn doors_stats_changed(
    query: Query<(Entity, DoorQuery), Changed<VitalsStats>>,
    mut commands: Commands,
) {
    for (entity, door) in query.iter() {
        if let map::ZoneKind::Door(door_kind) = door.zone.kind() {
            let is_enemy = true; // TODO: Check if door is_enemy, now just consider all doors as enemy

            commands.trigger_targets(
                GameServerPacket::from(DoorStatusUpdate::new(
                    door_kind,
                    *door.object_id,
                    door.vitals.as_ref(),
                    *door.status,
                    is_enemy,
                )),
                entity,
            );
        }
    }
}

fn on_required_components_changed(mut args: StatsCalcParams<VitalsStats>) -> Result<()> {
    for entity in args.calc_components_changed.iter() {
        if let Ok((stats_query, mut self_stats, in_world)) = args.query.get_mut(entity) {
            if stats_query.character && in_world.is_none() {
                continue;
            }
            let base_stats = if !stats_query.character {
                let npc_model = args.npc_info.get(entity)?;
                Some(&npc_model.stats.vitals)
            } else {
                let Some(sub_class) = stats_query.sub_class.as_deref() else {
                    continue;
                };
                args.stats_table
                    .vitals_stats(sub_class.class_id(), stats_query.progress_level.level())
            };
            let params = StatsCalculateParams::from_query(&stats_query, &args.formula_registry);
            let changed = self_stats.calculate(params, base_stats.map(|stats| stats.current()));
            if changed.is_some() && stats_query.character {
                args.user_info_updated.write(UserInfoUpdated(entity));
            }
        }
    }
    Ok(())
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut stats_table: ResMut<StatsTable>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let mut vitals_table = VitalsStatsHandlers::from(HashMap::new());

    for class_id in ClassId::iter() {
        let mut path = PathBuf::from("vitals_stats");
        path.push(CHRONICLE);
        path.push(format!("{class_id}"));
        path.set_extension("json");
        let handle: Handle<LeveledVitalsStats> = asset_server.load(path.clone());
        vitals_table.insert(class_id, handle);
    }
    stats_table.vitals_stats = vitals_table;
    *loaded = true;
}

fn update_stats_table(
    mut stats_table: ResMut<StatsTable>,
    mut events: EventReader<AssetEvent<LeveledVitalsStats>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id: _ } = event {
            stats_table.init_vitals_stats();
        }
    }
}

fn full_restore_trigger_handle(
    full_restore: Trigger<FullVitalsRestore>,
    mut full_restorers: Query<Mut<VitalsStats>>,
) -> Result<()> {
    let entity = full_restore.target();
    full_restorers.get_mut(entity)?.fill_current_from_max();
    Ok(())
}

fn full_restore_event_handle(
    mut full_restore: EventReader<FullVitalsRestore>,
    mut full_restorers: Query<Mut<VitalsStats>>,
) -> Result<()> {
    for event in full_restore.read() {
        debug!("Entity {:?} triggered full restore", event.entity());
        let entity = event.entity();
        full_restorers.get_mut(entity)?.fill_current_from_max();
    }
    Ok(())
}

fn resurrect_handle(
    ressurect: Trigger<Resurrect>,
    mut ressurectors: Query<Ref<ObjectId>>,
    mut commands: Commands,
) -> Result<()> {
    let entity = ressurect.target();
    let object_id = ressurectors.get_mut(entity)?;
    commands.trigger_targets(FullVitalsRestore::from(entity), entity);
    commands.entity(entity).remove::<(Dead, DeadTimer)>();
    commands.trigger_targets(
        ServerPacketBroadcast::new(Revive::new(*object_id).into()),
        entity,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::{assets::ASSET_DIR, utils::get_base_path};
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_single_vitals_stats() {
        let mut path = get_base_path();
        path.push(ASSET_DIR);
        path.push("vitals_stats");
        path.push(CHRONICLE);
        path.push("Bladedancer.json");

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
        let reader = BufReader::new(file);

        let vitals_stats: LeveledVitalsStats = from_reader(reader)
            .unwrap_or_else(|e| panic!("Failed to parse vitals stats from {:?}: {}", path, e));

        assert!(!vitals_stats.is_empty());

        // Check that level 1 stats exist and have valid values
        if let Some(level_1_stats) = vitals_stats.get(&Level::from(1)) {
            println!("Level 1 stats found: {:?}", level_1_stats);
            let max_hp = level_1_stats.get(VitalsStat::MaxHp);
            let max_mp = level_1_stats.get(VitalsStat::MaxMp);
            let max_cp = level_1_stats.get(VitalsStat::MaxCp);

            info!(
                "Level 1 Bladedancer stats: MaxHp={}, MaxMp={}, MaxCp={}",
                max_hp, max_mp, max_cp
            );

            assert!(
                max_hp > 0.0,
                "MaxHp should be greater than 0, got {}",
                max_hp
            );
            assert!(
                max_mp > 0.0,
                "MaxMp should be greater than 0, got {}",
                max_mp
            );
            assert!(
                max_cp > 0.0,
                "MaxCp should be greater than 0, got {}",
                max_cp
            );
        } else {
            panic!("Level 1 stats not found");
        }
    }

    #[test]
    fn test_parse_all_vitals_stats() {
        let mut vitals_dir = get_base_path();
        vitals_dir.push(ASSET_DIR);
        vitals_dir.push("vitals_stats");
        vitals_dir.push(CHRONICLE);

        let mut files_processed = 0;
        for entry in std::fs::read_dir(&vitals_dir)
            .unwrap_or_else(|_| panic!("Failed to read vitals_stats directory: {:?}", vitals_dir))
        {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file =
                    File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
                let reader = BufReader::new(file);

                let vitals_stats: LeveledVitalsStats = from_reader(reader).unwrap_or_else(|e| {
                    panic!("Failed to parse vitals stats from {:?}: {}", path, e)
                });

                assert!(
                    !vitals_stats.is_empty(),
                    "Vitals stats should not be empty for {:?}",
                    path
                );

                // Check level 1 stats for each class
                if let Some(level_1_stats) = vitals_stats.get(&Level::from(1)) {
                    let max_hp = level_1_stats.get(VitalsStat::MaxHp);
                    let max_mp = level_1_stats.get(VitalsStat::MaxMp);
                    let max_cp = level_1_stats.get(VitalsStat::MaxCp);

                    assert!(
                        max_hp > 0.0,
                        "{:?}: MaxHp should be > 0, got {}",
                        path.file_name(),
                        max_hp
                    );
                    assert!(
                        max_mp > 0.0,
                        "{:?}: MaxMp should be > 0, got {}",
                        path.file_name(),
                        max_mp
                    );
                    assert!(
                        max_cp > 0.0,
                        "{:?}: MaxCp should be > 0, got {}",
                        path.file_name(),
                        max_cp
                    );
                }

                files_processed += 1;
            }
        }

        assert!(files_processed > 0, "No vitals stats files were processed");
        info!(
            "Successfully processed {} vitals stats files",
            files_processed
        );
    }
}
