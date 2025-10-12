use bevy::prelude::*;
use game_core::{
    abnormal_effects::AbnormalEffects, network::packets::server::UserInfoUpdated, stats::*,
};
use state::StatKindSystems;

pub struct OtherStatsPlugin;
impl Plugin for OtherStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OtherStatsComponentsPlugin);

        app.add_systems(
            Update,
            on_required_components_changed.in_set(StatKindSystems::Other),
        );
        app.add_observer(update_abnormal_slots);
    }
}

fn on_required_components_changed(
    mut commands: Commands,
    mut args: StatsCalcParams<OtherStats>,
) -> Result<()> {
    for entity in args.calc_components_changed.iter() {
        let (stats_query, mut self_stats, in_world) = args.query.get_mut(entity)?;
        if stats_query.character && in_world.is_none() {
            continue;
        }
        let changed = self_stats.calculate(
            StatsCalculateParams::from_query(&stats_query, args.formula_registry.as_ref()),
            None,
        );
        if let Some(changed_stats) = changed {
            if stats_query.character {
                args.user_info_updated.write(UserInfoUpdated(entity));
            }
            let slot_changed = changed_stats.iter().any(|stat| stat.buff_slot_changed());
            if slot_changed {
                commands.trigger_targets(UpdateAbnormalSlots, entity);
            }
        }
    }
    Ok(())
}

fn update_abnormal_slots(
    trigger: Trigger<UpdateAbnormalSlots>,
    mut query: Query<(Mut<AbnormalEffects>, Ref<OtherStats>)>,
) -> Result<()> {
    let entity = trigger.target();
    let (mut abnormal_effects, self_stats) = query.get_mut(entity)?;

    abnormal_effects.set_max_buffs(self_stats.get(&OtherStat::MaxBuffSlots) as usize);
    abnormal_effects.set_max_debuffs(self_stats.get(&OtherStat::MaxDebuffSlots) as usize);
    abnormal_effects.set_max_rhythm(self_stats.get(&OtherStat::MaxRhythmSlots) as usize);

    Ok(())
}
