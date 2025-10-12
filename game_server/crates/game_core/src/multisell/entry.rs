use crate::multisell::Good;
use bevy::prelude::*;

#[derive(Clone, Debug, Default, Reflect)]
pub struct Entry {
    pub stackable: bool,
    pub rewards: Vec<Good>,
    pub requirements: Vec<Good>,
}

impl Entry {
    pub fn new(stackable: bool, rewards: Vec<Good>, requirements: Vec<Good>) -> Self {
        Self {
            stackable,
            rewards,
            requirements,
        }
    }
}
