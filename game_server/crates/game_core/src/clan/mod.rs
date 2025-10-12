pub mod castle;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub type ClanLevel = u8;

#[derive(
    Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, Reflect,
)]
#[repr(u8)]
pub enum ClanRank {
    #[default]
    Vagabond,
    Vassal,
    Heir,
    Knight,
    Elder,
    Baron,
    Viscount,
    Count,
    Marquis,
    Duke,
    GrandDuke,
    King,
}
