use bevy::{platform::collections::HashMap, prelude::*};
use bevy_enum_tag::EnumComponentTag;
use l2r_core::model::race::Race;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

mod castle;
mod clan_hall;
mod door;
mod fort;
mod olympiad_stadium;
mod residence_hall_teleport;
mod residence_teleport;
mod respawn;
mod respawn_points;
mod siegable_hall;
mod spawn;
mod town;

pub use castle::*;
pub use clan_hall::*;
pub use door::*;
pub use fort::*;
pub use olympiad_stadium::*;
pub use residence_hall_teleport::*;
pub use residence_teleport::*;
pub use respawn::*;
pub use respawn_points::*;
pub use siegable_hall::*;
pub use spawn::*;
pub use town::*;
pub use zone_kind_variant::*;

// Race and fallback location name
#[derive(Clone, Debug, Deref, Deserialize, Reflect, Serialize)]
pub struct BannedRaces(HashMap<Race, String>);

#[derive(Component)]
pub struct ZoneKindFolder;

#[derive(Clone, Debug, Default, Deserialize, EnumDiscriminants, Reflect, Serialize)]
#[strum_discriminants(name(ZoneKindVariant))]
#[strum_discriminants(derive(Display, EnumString, EnumIter, EnumComponentTag, Hash, Reflect))]
#[strum_discriminants(strum(serialize_all = "title_case"))]
pub enum ZoneKind {
    Arena,
    Boss,
    Castle(CastleKind),
    ClanHall(ClanHallKind),
    Condition,
    Damage,
    DerbyTrack,
    Door(DoorKind),
    Effect,
    Fishing,
    Fort(FortKind),
    Hq,
    Jail,
    Landing,
    #[default]
    Other,
    MotherTree,
    NoLanding,
    NoRestart,
    NoStore,
    NoSummonFriend,
    OlympiadStadium(OlympiadStadiumKind),
    Peace,
    ResidenceHallTeleport(ResidenceHallTeleportKind),
    ResidenceTeleport(ResidenceTeleportKind),
    Residence,
    RespawnPoints(RespawnPointsKind),
    Respawn(RespawnKind),
    Script,
    SiegableHall(SiegableHallKind),
    Siege,
    Swamp,
    Spawn(SpawnKind),
    Town(TownKind),
    Water,
}

pub trait AlwaysLoadedZones {
    fn name() -> &'static str;
}
