use super::monster_ai::MonsterAiParams;
use crate::{items::DropTable, npc::id::DisplayId, stats::*};
use bevy::{platform::collections::HashMap, prelude::*};
use l2r_core::model::race::Race;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Model {
    pub display_id: Option<DisplayId>,
    pub level: Level,
    pub kind: super::Kind,
    pub name: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub race: Race,
    #[serde(default)]
    pub gender: Gender,
    pub reward: Option<ProgressReward>,
    pub stats: Stats,
    pub skill_list: Option<Vec<SkillInfo>>,
    pub ai: Option<MonsterAiParams>,
    pub drop_table: Option<DropTable>,
    #[serde(default)]
    pub collision: CollisionSize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillInfo {
    pub id: u32,
    pub level: u32,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Stats {
    pub primal: PrimalStats,
    pub vitals: VitalsStats,
    #[serde(default)]
    pub attack: AttackStats,
    #[serde(default)]
    pub critical: CriticalStats,
    #[serde(default)]
    pub defence: DefenceStats,
    #[serde(default)]
    pub attribute: Attribute,
    #[serde(default)]
    pub speed: MovementStats,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Attribute {
    pub defence: HashMap<Element, f32>,
}

#[derive(Clone, Component, Copy, Debug, Default, Deserialize)]
pub struct CollisionSize {
    normal: ColliderInfo,
    grown: Option<ColliderInfo>,
}
impl CollisionSize {
    pub fn get(&self, collider_size: ColliderSize) -> ColliderInfo {
        match collider_size {
            ColliderSize::Normal => self.normal,
            ColliderSize::Grown => self.grown.unwrap_or(self.normal),
        }
    }
}
