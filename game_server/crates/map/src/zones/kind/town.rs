use super::BannedRaces;
use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct TownKind {
    banned_races: Option<BannedRaces>,
    spawn_points: SpawnPoints,
}
impl TownKind {
    pub fn banned_races(&self) -> &Option<BannedRaces> {
        &self.banned_races
    }
}
impl SpawnPointsGetter for TownKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct TownZonesList(ZoneListHandle);

impl From<ZoneListHandle> for TownZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for TownZonesList {
    fn name() -> &'static str {
        "town"
    }
}

impl AsRef<ZoneListHandle> for TownZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
