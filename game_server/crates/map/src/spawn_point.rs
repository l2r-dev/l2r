use bevy::prelude::*;
use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use spatial::GameVec3;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum SpawnPointKind {
    #[default]
    Other,
    Chaotic,
    Banish,
    Challenger,
    SpectatorSpawn,
    Entrance,
}

#[derive(Clone, Copy, Debug, Deserialize, Reflect, Serialize)]
pub struct SpawnPoint {
    #[serde(flatten)]
    pub loc: GameVec3,
    #[serde(default)]
    pub kind: SpawnPointKind,
}
impl Default for SpawnPoint {
    // Talking Island Village
    fn default() -> Self {
        Self {
            loc: GameVec3::new(-83761, 243620, -3700),
            kind: SpawnPointKind::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut, Deserialize, Reflect, Serialize)]
pub struct SpawnPoints(Vec<SpawnPoint>);
impl SpawnPoints {
    pub fn random(&self) -> SpawnPoint {
        let rng = &mut rand::thread_rng();
        self.choose(rng).copied().unwrap_or_default()
    }
    pub fn random_typed(&self, kind: SpawnPointKind) -> SpawnPoint {
        let rng = &mut rand::thread_rng();
        self.iter()
            .filter(|p| p.kind == kind)
            .choose(rng)
            .copied()
            .unwrap_or_default()
    }
}

impl From<SpawnPoint> for Vec3 {
    fn from(spawn_point: SpawnPoint) -> Self {
        spawn_point.loc.into()
    }
}

impl From<SpawnPoint> for GameVec3 {
    fn from(spawn_point: SpawnPoint) -> Self {
        spawn_point.loc
    }
}

impl From<SpawnPoints> for Vec<Vec3> {
    fn from(spawn_points: SpawnPoints) -> Self {
        spawn_points.iter().map(|p| (*p).into()).collect()
    }
}

pub trait SpawnPointsGetter {
    fn spawn_points(&self) -> &SpawnPoints;
}
