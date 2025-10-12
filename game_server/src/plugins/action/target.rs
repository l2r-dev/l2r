use bevy::prelude::*;
use game_core::action::target::TargetComponentsPlugin;

pub struct TargetPlugin;
impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TargetComponentsPlugin);
    }
}
