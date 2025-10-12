use bevy::prelude::*;
use game_core::{
    attack::Dead,
    encounters::EnteredWorld,
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{
            GameServerPacket, Social, SocialAction, StatusUpdate, StatusUpdateKind, SystemMessage,
            UserInfoUpdated,
        },
    },
    object_id::ObjectId,
    stats::{
        FullVitalsRestore, ProgressLevelStats, ProgressRatesStats, ProgressStat, ProgressStats,
        ProgressStatsComponentsPlugin, Stats, StatsCalcParams, StatsCalculateParams,
    },
};
use state::GameMechanicsSystems;
use system_messages::Id;

pub struct ProgressStatsPlugin;
impl Plugin for ProgressStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProgressStatsComponentsPlugin);

        app.add_systems(
            Update,
            changed_progress_rates_stats
                .in_set(GameMechanicsSystems::StatsCalculation)
                .before(changed_progress_stats),
        )
        .add_systems(
            Update,
            (changed_progress_stats, changed_progress_level_stats)
                .chain()
                .in_set(GameMechanicsSystems::StatsCalculation),
        );
    }
}

fn changed_progress_stats(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Ref<ObjectId>,
            Ref<ProgressStats>,
            Mut<ProgressLevelStats>,
        ),
        (Changed<ProgressStats>, With<EnteredWorld>),
    >,
    mut user_info_updates: EventWriter<UserInfoUpdated>,
) {
    for (entity, object_id, progress_stats, mut progress_level_stats) in query.iter_mut() {
        let mut status_update = StatusUpdate::new(*object_id);
        for (stat, _) in progress_stats.iter() {
            match stat {
                ProgressStat::Exp => {
                    let current_level = progress_level_stats.level();
                    let new_level = progress_stats.calculate_level_by_exp();
                    if new_level != current_level {
                        progress_level_stats.set_level(new_level);
                    }

                    let exp = progress_stats.exp();
                    status_update.add(StatusUpdateKind::Exp, exp as u32);
                }
                ProgressStat::Sp => {
                    let sp = progress_stats.sp();
                    status_update.add(StatusUpdateKind::Sp, sp);
                }
                _ => {}
            }
        }
        commands.trigger_targets(GameServerPacket::from(status_update), entity);
        user_info_updates.write(UserInfoUpdated(entity));
    }
}

fn changed_progress_level_stats(
    mut commands: Commands,
    mut query: Query<
        (Entity, Ref<ObjectId>, Ref<ProgressLevelStats>, Has<Dead>),
        (Changed<ProgressLevelStats>, With<EnteredWorld>),
    >,
    mut full_vitals_restore_events: EventWriter<FullVitalsRestore>,
) {
    for (entity, object_id, progress_level_stats, dead) in query.iter_mut() {
        let mut status_update = StatusUpdate::new(*object_id);
        let level = progress_level_stats.level();
        let prev_level = progress_level_stats.prev_level();

        // Only send level up action and messages if the new level is greater than previous
        if level > prev_level {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new_empty(Id::YourLevelHasIncreased)),
                entity,
            );
            let level_up_action = SocialAction::new(*object_id, Social::LevelUp);
            commands.trigger_targets(ServerPacketBroadcast::new(level_up_action.into()), entity);
        }

        bevy::log::debug!(
            "Entity {:?} changed to level {} (previous level was {}), dead: {}",
            entity,
            level,
            prev_level,
            dead
        );

        if !dead {
            full_vitals_restore_events.write(FullVitalsRestore::from(entity));
        }

        status_update.add(StatusUpdateKind::Level, level.into());
        commands.trigger_targets(GameServerPacket::from(status_update), entity);
    }
}

fn changed_progress_rates_stats(mut args: StatsCalcParams<ProgressRatesStats>) -> Result<()> {
    for entity in args.calc_components_changed.iter() {
        if let Ok((stats_query, mut self_stats, in_world)) = args.query.get_mut(entity) {
            if stats_query.character && in_world.is_none() {
                continue;
            }
            let params =
                StatsCalculateParams::from_query(&stats_query, args.formula_registry.as_ref());
            let changed = self_stats.calculate(params, None);
            if changed.is_some() && stats_query.character {
                args.user_info_updated.write(UserInfoUpdated(entity));
            }
        }
    }
    Ok(())
}
