use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub type PvpFlag = bool;
pub type Karma = u32;
pub type PvpKills = u32;
pub type PkKills = u32;

#[derive(Clone, Component, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct PvpStats {
    pub pvp_flag: PvpFlag,
    pub karma: Karma,
    pub pk_kills: PvpKills,
    pub pvp_kills: PkKills,
}
