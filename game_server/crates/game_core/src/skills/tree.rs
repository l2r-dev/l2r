use crate::{
    items,
    stats::{self, ClassId},
};
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use derive_more::From;
use serde::{Deserialize, Serialize};

pub struct SkillTreesComponentsPlugin;
impl Plugin for SkillTreesComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<SkillTree>::new(&["json"]));

        app.register_asset_reflect::<SkillTree>()
            .register_type::<SkillTreeNode>()
            .register_type::<LearnRequirements>();
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Reflect, Serialize)]
pub enum LearnRequirement {
    Auto,
    Level(stats::Level),
    Sp(stats::Sp),
    Item((items::Id, u64)),
}

#[derive(Clone, Debug, Deserialize, Reflect)]
pub struct LearnRequirements(pub Vec<LearnRequirement>);

#[derive(Clone, Debug, Deserialize, Reflect)]
pub struct SkillTreeNode {
    pub skill_id: super::Id,
    pub skill_level: super::Level,
    pub requirements: LearnRequirements,
}

impl From<&SkillTreeNode> for super::Skill {
    fn from(node: &SkillTreeNode) -> Self {
        Self::new(node.skill_id, node.skill_level)
    }
}

#[derive(Asset, Clone, Debug, Default, Deref, Deserialize, Reflect)]
pub struct SkillTree(Vec<SkillTreeNode>);
impl SkillTree {
    pub fn skills_on_level(&self, level: stats::Level) -> Vec<&SkillTreeNode> {
        self.0
            .iter()
            .filter(|node| {
                node.requirements
                    .0
                    .iter()
                    .any(|req| matches!(req, LearnRequirement::Level(l) if *l == level))
            })
            .collect()
    }

    pub fn auto_skill_on_level(&self, level: stats::Level) -> Vec<SkillTreeNode> {
        self.0
            .iter()
            .filter(|node| {
                let mut has_auto = false;
                let mut has_level = false;

                for req in node.requirements.0.iter() {
                    match req {
                        LearnRequirement::Auto => has_auto = true,
                        LearnRequirement::Level(l) if *l == level => has_level = true,
                        _ => {}
                    }

                    if has_auto && has_level {
                        return true;
                    }
                }

                false
            })
            .cloned()
            .collect()
    }
}

#[derive(Clone, Default, Deref, DerefMut, From, Reflect, Resource)]
#[reflect(Resource)]
pub struct SkillTreesHandlers(HashMap<ClassId, Handle<SkillTree>>);

impl SkillTreesHandlers {
    pub fn get_data<'a>(
        &self,
        class_id: ClassId,
        skill_trees: &'a Assets<SkillTree>,
    ) -> Result<&'a SkillTree> {
        let handle = self.get(&class_id).ok_or_else(|| {
            BevyError::from(format!("Skill tree with class ID {} not found", class_id))
        })?;
        skill_trees.get(handle).ok_or_else(|| {
            BevyError::from(format!(
                "Skill tree asset not found for handle: {:?}",
                handle
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn test_data_json() -> &'static str {
        r#"[
            {
                "skill_id": 1,
                "skill_level": 1,
                "requirements": ["Auto", {"Level": 1}]
            },
            {
                "skill_id": 2,
                "skill_level": 2,
                "requirements": ["Auto", {"Level": 1}]
            },
            {
                "skill_id": 3,
                "skill_level": 1,
                "requirements": [{"Sp": 100}]
            },
            {
                "skill_id": 4,
                "skill_level": 3,
                "requirements": [{"Item": [42, 5]}]
            },
            {
                "skill_id": 5,
                "skill_level": 1,
                "requirements": [
                    {"Level": 1},
                    {"Sp": 50},
                    {"Item": [123, 1]}
                ]
            }
        ]"#
    }

    #[test]
    fn test_skill_tree_deserialize_json() {
        let skill_tree: SkillTree =
            serde_json::from_str(test_data_json()).expect("Failed to deserialize SkillTree");

        assert_eq!(skill_tree.0.len(), 5);
    }

    #[test]
    fn test_skill_tree_skills_on_level() {
        let skill_tree: SkillTree =
            serde_json::from_str(test_data_json()).expect("Failed to deserialize SkillTree");

        let skills = skill_tree.skills_on_level(1.into());
        assert_eq!(skills.len(), 3);
    }

    #[test]
    fn test_skill_tree_skills_without_level_requirements() {
        let skill_tree: SkillTree =
            serde_json::from_str(test_data_json()).expect("Failed to deserialize SkillTree");

        let skills = skill_tree.auto_skill_on_level(1.into());
        assert_eq!(skills.len(), 2);
    }
}
