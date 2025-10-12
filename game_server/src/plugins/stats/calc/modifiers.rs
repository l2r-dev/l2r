use bevy::prelude::*;
use game_core::stats::StatModifiersComponentsPlugin;

pub(super) struct StatModifiersPlugin;
impl Plugin for StatModifiersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatModifiersComponentsPlugin);
    }
}
