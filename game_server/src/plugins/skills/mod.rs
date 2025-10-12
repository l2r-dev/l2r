use bevy::prelude::*;
use game_core::skills::{SkillReuseTimerPlugin, SkillsComponentsPlugin};

mod trees;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SkillsComponentsPlugin);
        app.add_plugins(SkillReuseTimerPlugin);
        app.add_plugins(trees::SkillTreesPlugin);

        app.register_type::<game_core::skills::SkillList>();
    }
}
