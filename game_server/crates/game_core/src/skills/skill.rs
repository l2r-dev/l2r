use crate::items::ItemSkill;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Skill {
    id: super::Id,
    display_id: Option<super::Id>,
    level: super::Level,
    magic_level: u32,
    kind: super::Kind,
    disabled: bool,
}

impl PartialEq for Skill {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.level == other.level
    }
}
impl Eq for Skill {}

use std::hash::{Hash, Hasher};
impl Hash for Skill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.level.hash(state);
    }
}

impl Skill {
    pub fn new(id: super::Id, level: super::Level) -> Self {
        Self {
            id,
            level,
            ..Default::default()
        }
    }

    pub fn enable(&mut self) {
        self.disabled = false;
    }

    pub fn disable(&mut self) {
        self.disabled = true;
    }

    pub fn passive(&self) -> bool {
        matches!(self.kind, super::Kind::Passive | super::Kind::Toggle)
    }

    pub fn set_passive(&mut self) {
        self.kind = super::Kind::Passive;
    }

    pub fn enchanted(&self) -> bool {
        *self.level > 100
    }

    pub fn id(&self) -> super::Id {
        self.id
    }

    pub fn display_id(&self) -> super::Id {
        self.display_id.unwrap_or(self.id)
    }

    pub fn level(&self) -> super::Level {
        self.level
    }

    pub fn magic_level(&self) -> u32 {
        self.magic_level
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }
}

impl From<ItemSkill> for Skill {
    fn from(item_skill: ItemSkill) -> Self {
        Self {
            id: item_skill.id.into(),
            level: item_skill.level.into(),
            ..Default::default()
        }
    }
}
