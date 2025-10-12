use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct FortKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for FortKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct FortZonesList(ZoneListHandle);

impl From<ZoneListHandle> for FortZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for FortZonesList {
    fn name() -> &'static str {
        "fort"
    }
}

impl AsRef<ZoneListHandle> for FortZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
