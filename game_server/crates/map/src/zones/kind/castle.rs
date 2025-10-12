use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct CastleKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for CastleKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct CastleZonesList(ZoneListHandle);

impl From<ZoneListHandle> for CastleZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for CastleZonesList {
    fn name() -> &'static str {
        "castle"
    }
}

impl AsRef<ZoneListHandle> for CastleZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
