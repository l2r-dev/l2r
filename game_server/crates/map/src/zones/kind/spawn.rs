use crate::zones::Zone;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct SpawnKind {
    #[reflect(ignore)]
    pub banned_zone: Option<Box<Zone>>,
}
impl SpawnKind {
    pub fn banned_zone(&self) -> Option<&Zone> {
        self.banned_zone.as_ref().map(|z| z.as_ref())
    }
}
