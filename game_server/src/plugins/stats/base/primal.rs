use bevy::prelude::*;
use game_core::{network::packets::server::UserInfoUpdated, npc::RegionalNpcInfoQuery, stats::*};
use l2r_core::model::race::Race;
use state::StatKindSystems;

pub struct PrimalStatsPlugin;
impl Plugin for PrimalStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PrimalStatsComponentsPlugin);
        app.add_systems(
            Update,
            on_required_components_changed.in_set(StatKindSystems::Primal),
        );
    }
}

fn on_required_components_changed(
    components_changed: Query<
        Entity,
        Or<(
            Changed<StatModifiers>,
            Changed<ProgressLevelStats>,
            Changed<Race>,
        )>,
    >,
    mut user_info_updated: EventWriter<UserInfoUpdated>,
    formula_registry: Res<StatFormulaRegistry>,
    stats_table: StatsTableQuery,
    npc_info: RegionalNpcInfoQuery,
    mut query: Query<(Ref<Race>, StatsCalcNoPrimalQuery, Mut<PrimalStats>)>,
) -> Result<()> {
    let race_stats = stats_table.race_stats();
    for entity in components_changed.iter() {
        if let Ok((race, stats_query, mut self_stats)) = query.get_mut(entity) {
            let base_class_stats = race_stats.get(*race, *stats_query.base_class);
            let base_stats = if !stats_query.character {
                let npc_model = npc_info.get(entity)?;
                &npc_model.stats.primal
            } else {
                &base_class_stats.primal_stats
            };

            let params = StatsCalculateParams::from_query_no_primal(
                &stats_query,
                base_stats,
                &formula_registry,
            );
            let changed = self_stats.calculate(params, Some(base_stats.as_ref()));
            if changed.is_some() && stats_query.character {
                user_info_updated.write(UserInfoUpdated(entity));
            }
        }
    }
    Ok(())
}
