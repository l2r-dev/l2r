use bevy::prelude::*;
use game_core::{network::packets::server::UserInfoUpdated, stats::*};
use state::StatKindSystems;

pub struct MovementStatsPlugin;
impl Plugin for MovementStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MovementStatsComponentsPlugin);
        app.add_systems(
            Update,
            on_required_components_changed.in_set(StatKindSystems::Movement),
        );
    }
}

fn on_required_components_changed(mut args: StatsCalcParams<Movable>) -> Result<()> {
    for entity in args.calc_components_changed.iter() {
        let race_stats = args.stats_table.race_stats();
        if let Ok((stats_query, mut self_stats, in_world)) = args.query.get_mut(entity) {
            if stats_query.character && in_world.is_none() {
                continue;
            }
            let base_class_stats = race_stats.get(*stats_query.race, *stats_query.base_class);
            let base_stats = if !stats_query.character {
                let npc_model = args.npc_info.get(entity)?;
                &npc_model.stats.speed
            } else {
                &base_class_stats.base_speed
            };
            let params =
                StatsCalculateParams::from_query(&stats_query, args.formula_registry.as_ref());
            let changed = self_stats
                .movement_stats_mut()
                .calculate(params, Some(base_stats.as_ref()));
            if changed.is_some() && stats_query.character {
                args.user_info_updated.write(UserInfoUpdated(entity));
            }
        }
    }
    Ok(())
}
