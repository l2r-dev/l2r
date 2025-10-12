use super::BannedRaces;
use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct RespawnPointsKind {
    banned_races: BannedRaces,
    spawn_points: SpawnPoints,
}
impl RespawnPointsKind {
    pub fn banned_races(&self) -> &BannedRaces {
        &self.banned_races
    }
}
impl SpawnPointsGetter for RespawnPointsKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct RespawnPointsZonesList(ZoneListHandle);

impl From<ZoneListHandle> for RespawnPointsZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for RespawnPointsZonesList {
    fn name() -> &'static str {
        "respawn_points"
    }
}

impl AsRef<ZoneListHandle> for RespawnPointsZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
