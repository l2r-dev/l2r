use bevy::ecs::{entity::Entity, event::Event};
use bevy_ecs::entity::ContainsEntity;

#[derive(Clone, Copy, Debug, Event)]
pub struct UseShot {
    entity: Entity,
    shot_entity: Entity,
}

impl UseShot {
    pub fn new(entity: Entity, shot_entity: Entity) -> Self {
        Self {
            entity,
            shot_entity,
        }
    }

    pub fn shot_entity(&self) -> Entity {
        self.shot_entity
    }
}

impl ContainsEntity for UseShot {
    fn entity(&self) -> Entity {
        self.entity
    }
}
