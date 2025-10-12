use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct ResidenceTeleportKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for ResidenceTeleportKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct ResidenceTeleportZonesList(ZoneListHandle);

impl From<ZoneListHandle> for ResidenceTeleportZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for ResidenceTeleportZonesList {
    fn name() -> &'static str {
        "residence_teleport"
    }
}

impl AsRef<ZoneListHandle> for ResidenceTeleportZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
