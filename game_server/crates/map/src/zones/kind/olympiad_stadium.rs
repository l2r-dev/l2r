use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct OlympiadStadiumKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for OlympiadStadiumKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct OlympiadStadiumZonesList(ZoneListHandle);

impl From<ZoneListHandle> for OlympiadStadiumZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for OlympiadStadiumZonesList {
    fn name() -> &'static str {
        "olympiad_stadium"
    }
}

impl AsRef<ZoneListHandle> for OlympiadStadiumZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
