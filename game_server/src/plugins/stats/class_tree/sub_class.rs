use bevy::prelude::*;
use game_core::stats::{StatsTableQuery, SubClass};
use state::GameMechanicsSystems;

pub struct SubClassStatsPlugin;
impl Plugin for SubClassStatsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SubClass>();

        app.add_systems(
            Update,
            update_changed_base_class.in_set(GameMechanicsSystems::StatsCalculation),
        );
    }
}

fn update_changed_base_class(
    mut command: Commands,
    stats_table: StatsTableQuery,
    sub_classes: Query<(Entity, Ref<SubClass>), Changed<SubClass>>,
) {
    for (entity, sub_class) in sub_classes.iter() {
        let base_class = stats_table
            .class_tree()
            .get_base_class(sub_class.class_id());
        command.entity(entity).try_insert(base_class);
    }
}
