use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
pub struct RespawnKind {
    target_zone_name: String,
    target_entity: Option<Entity>,
}
impl RespawnKind {
    pub fn name(&self) -> &str {
        &self.target_zone_name
    }
    pub fn set_target_entity(&mut self, entity: Entity) {
        self.target_entity = Some(entity);
    }
    pub fn target_entity(&self) -> Option<Entity> {
        self.target_entity
    }
}
