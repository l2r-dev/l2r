use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct SiegableHallKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for SiegableHallKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct SiegableHallZonesList(ZoneListHandle);

impl From<ZoneListHandle> for SiegableHallZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for SiegableHallZonesList {
    fn name() -> &'static str {
        "siegable_hall"
    }
}

impl AsRef<ZoneListHandle> for SiegableHallZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
