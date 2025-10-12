use bevy::{platform::collections::HashMap, prelude::*};
use game_core::{
    skills::{SkillTreesComponentsPlugin, SkillTreesHandlers},
    stats::ClassId,
};
use l2r_core::chronicles::CHRONICLE;
use std::path::PathBuf;
use strum::IntoEnumIterator;

pub struct SkillTreesPlugin;

impl Plugin for SkillTreesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SkillTreesComponentsPlugin);

        app.add_systems(Startup, load_assets);
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut skill_trees = SkillTreesHandlers::from(HashMap::new());
    for class_id in ClassId::iter() {
        let mut path = PathBuf::from("skills_trees");
        path.push(CHRONICLE);
        path.push(format!("{class_id}"));
        path.set_extension("json");
        skill_trees.insert(class_id, asset_server.load(path.clone()));
    }
    commands.insert_resource(skill_trees);
}
