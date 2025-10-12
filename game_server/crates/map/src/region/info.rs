use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Asset, Debug, Deserialize, Reflect, Serialize)]
pub struct RegionInfo {
    respawn_zone: String,
}
impl RegionInfo {
    pub fn respawn_zone(&self) -> &str {
        &self.respawn_zone
    }
}

#[derive(Clone, Component, Copy, Debug, Reflect)]
pub struct RegionRespawnZone(pub Entity);

#[derive(Clone, Component, Default, Deref)]
pub struct RegionInfoHandle(Handle<RegionInfo>);
impl From<Handle<RegionInfo>> for RegionInfoHandle {
    fn from(handle: Handle<RegionInfo>) -> Self {
        Self(handle)
    }
}
