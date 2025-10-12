use crate::npc;
use bevy::{prelude::*, reflect::Reflect};
use map::Zone;
use rand::Rng;
use serde::{Deserialize, Deserializer};
use spatial::{GameVec3, Heading, calculate_centroid};
use std::time::Duration;

#[derive(Deserialize)]
struct SpawnerTimerInfo {
    delay: u32,
    #[serde(default)]
    random: Option<u32>,
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct SpawnerTimer {
    #[deref]
    timer: Timer,
    default_duration: Duration,
    random: Option<u32>,
}
impl SpawnerTimer {
    pub fn new(seconds: u32, random: Option<u32>) -> Self {
        let seconds = seconds as u64;
        let mut timer = Timer::new(Duration::from_secs(seconds), TimerMode::Once);
        // Setting This to elapsed to spawn NPC immediately, when creating spawner
        let duration = Duration::from_secs(seconds);
        timer.set_elapsed(duration);
        Self {
            timer,
            random,
            default_duration: duration,
        }
    }
    pub fn reset(&mut self) {
        if let Some(random) = self.random {
            let random = random as i32;
            let random_time = rand::thread_rng().gen_range(-random..random) as i64;
            let default_seconds = self.default_duration.as_secs() as i64;
            let seconds = (default_seconds + random_time).max(1) as u64;
            self.timer = Timer::new(Duration::from_secs(seconds), TimerMode::Once);
        }
        self.timer.reset();
    }
}
impl<'de> Deserialize<'de> for SpawnerTimer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let info = SpawnerTimerInfo::deserialize(deserializer)?;
        Ok(SpawnerTimer::new(info.delay, info.random))
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(Name::new("Spawners".to_string()))]
pub struct RegionalSpawners;

#[derive(Clone, Component, Debug, Default, Deserialize, Reflect)]
pub struct Spawner {
    pub name: Option<String>,
    pub zone: Option<Zone>,
    pub npcs: Vec<NpcSpawnInfo>,
}

impl Spawner {
    pub fn build(&self) -> (Self, Transform, Name) {
        let mut transform = Transform::default();
        let name = Name::new(format!(
            "spawner-{}",
            match &self.name {
                Some(name) => name.clone(),
                None => self
                    .npcs
                    .iter()
                    .map(|npc| npc.id.to_string())
                    .collect::<Vec<_>>()
                    .join("_"),
            }
        ));
        if let Some(zone) = &self.zone {
            let (zone_transform, _, _, _, _) = zone.build();
            transform = zone_transform;
        } else {
            let locs = self
                .npcs
                .iter()
                .filter_map(|npc| npc.loc)
                .map(|loc| loc.into())
                .collect::<Vec<_>>();
            let centroid = calculate_centroid(locs.as_slice());
            if let Some(centroid) = centroid {
                transform = Transform::from_translation(centroid);
            }
        }
        (self.clone(), transform, name)
    }
    pub fn npc(&self, id: npc::Id) -> Option<&NpcSpawnInfo> {
        self.npcs.iter().find(|npc| npc.id == id)
    }
    pub fn npc_mut(&mut self, id: npc::Id) -> Option<&mut NpcSpawnInfo> {
        self.npcs.iter_mut().find(|npc| npc.id == id)
    }
}

#[derive(Component, Reflect)]
pub struct SpawnZone;

#[derive(Component, Reflect)]
pub struct BannedSpawnZone;

#[derive(Component, Reflect)]
#[require(Name::new("BannedSpawnZones".to_string()))]
pub struct BannedSpawnZones;

fn default_chase_range() -> u32 {
    1000
}
fn default_count() -> u32 {
    1
}
#[derive(Clone, Debug, Deserialize, Reflect)]
pub struct NpcSpawnInfo {
    pub(crate) id: npc::Id,
    #[serde(default = "default_count")]
    count: u32,
    #[serde(default)]
    count_alive: u32,
    loc: Option<GameVec3>,
    #[serde(default)]
    heading: Heading,
    timer: SpawnerTimer,
    #[serde(default = "default_chase_range")]
    chase_range: u32,
}
impl NpcSpawnInfo {
    pub fn id(&self) -> npc::Id {
        self.id
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn loc(&self) -> Option<GameVec3> {
        self.loc
    }
    pub fn transform(&self) -> Transform {
        let mut transform = Transform::default();
        if let Some(loc) = self.loc {
            transform = Transform::from_translation(loc.into());

            if *self.heading != 0 {
                transform.rotation = Quat::from(self.heading);
            } else {
                transform.rotation = Quat::from(Heading::random());
            }
            transform
        } else {
            transform.rotation = Quat::from(Heading::random());
            transform
        }
    }
    pub fn timer(&self) -> &SpawnerTimer {
        &self.timer
    }
    pub fn timer_mut(&mut self) -> &mut SpawnerTimer {
        &mut self.timer
    }
    pub fn chase_range(&self) -> u32 {
        self.chase_range
    }
    pub fn count_alive(&self) -> u32 {
        self.count_alive
    }
    pub fn needed(&self) -> u32 {
        self.count.saturating_sub(self.count_alive)
    }
    pub fn fullfilled(&self) -> bool {
        self.count_alive >= self.count
    }
    pub fn inc_count_alive(&mut self) {
        self.count_alive = self.count_alive.saturating_add(1);
    }
    pub fn dec_count_alive(&mut self) {
        self.count_alive = self.count_alive.saturating_sub(1);
    }
}

#[derive(Asset, Clone, Debug, Default, Deref, Deserialize, Resource, TypePath)]
pub struct SpawnList(Vec<Spawner>);

#[derive(Component, Default, Deref)]
pub struct SpawnListHandle(Handle<SpawnList>);
impl From<Handle<SpawnList>> for SpawnListHandle {
    fn from(handle: Handle<SpawnList>) -> Self {
        Self(handle)
    }
}
