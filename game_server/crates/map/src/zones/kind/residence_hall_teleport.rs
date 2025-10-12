use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct ResidenceHallTeleportKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for ResidenceHallTeleportKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct ResidenceHallTeleportZonesList(ZoneListHandle);

impl From<ZoneListHandle> for ResidenceHallTeleportZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for ResidenceHallTeleportZonesList {
    fn name() -> &'static str {
        "residence_hall_teleport"
    }
}

impl AsRef<ZoneListHandle> for ResidenceHallTeleportZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
