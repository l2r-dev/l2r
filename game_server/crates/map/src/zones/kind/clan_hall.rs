use crate::{SpawnPoints, SpawnPointsGetter, zones::ZoneListHandle};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct ClanHallKind {
    spawn_points: SpawnPoints,
}
impl SpawnPointsGetter for ClanHallKind {
    fn spawn_points(&self) -> &SpawnPoints {
        &self.spawn_points
    }
}

#[derive(Clone, Deref, Resource)]
pub struct ClanHallZonesList(ZoneListHandle);

impl From<ZoneListHandle> for ClanHallZonesList {
    fn from(handle: ZoneListHandle) -> Self {
        Self(handle)
    }
}

impl super::AlwaysLoadedZones for ClanHallZonesList {
    fn name() -> &'static str {
        "clan_hall"
    }
}

impl AsRef<ZoneListHandle> for ClanHallZonesList {
    fn as_ref(&self) -> &ZoneListHandle {
        &self.0
    }
}
