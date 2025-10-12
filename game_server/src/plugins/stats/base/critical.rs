use bevy::prelude::*;
use game_core::{
    network::packets::server::UserInfoUpdated,
    stats::{
        CriticalComponentsPlugin, CriticalStats, Stats, StatsCalcParams, StatsCalculateParams,
    },
};
use state::StatKindSystems;

pub struct CriticalStatsPlugin;
impl Plugin for CriticalStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CriticalComponentsPlugin);
        app.add_systems(
            Update,
            on_required_components_changed.in_set(StatKindSystems::Critical),
        );
    }
}

fn on_required_components_changed(mut args: StatsCalcParams<CriticalStats>) -> Result<()> {
    for entity in args.calc_components_changed.iter() {
        if let Ok((stats_query, mut self_stats, in_world)) = args.query.get_mut(entity) {
            if stats_query.character && in_world.is_none() {
                continue;
            }
            let base_stats = if !stats_query.character {
                let npc_model = args.npc_info.get(entity)?;
                Some(&npc_model.stats.critical)
            } else {
                None
            };

            let params =
                StatsCalculateParams::from_query(&stats_query, args.formula_registry.as_ref());
            let changed = self_stats.calculate(params, base_stats.map(|v| v.as_ref()));
            if changed.is_some() && stats_query.character {
                args.user_info_updated.write(UserInfoUpdated(entity));
            }
        }
    }
    Ok(())
}
