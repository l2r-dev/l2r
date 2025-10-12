use crate::skills::{self, Skill, SkillReuseTimers};
use bevy::{platform::collections::HashMap, prelude::*};
use core::fmt;

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
#[require(SkillReuseTimers)]
#[reflect(Component)]
pub struct SkillList(HashMap<skills::Id, Skill>);

impl fmt::Debug for SkillList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl SkillList {
    pub fn add_skill(&mut self, skill: Skill) -> &mut Self {
        self.0.insert(skill.id(), skill);
        self
    }

    pub fn get_passives(&self) -> Vec<Skill> {
        self.0
            .values()
            .filter(|skill| skill.passive())
            .cloned()
            .collect()
    }
}
