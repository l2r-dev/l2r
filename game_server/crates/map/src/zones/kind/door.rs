use bevy::{platform::collections::HashSet, prelude::*};
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    Deref,
    Eq,
    From,
    Hash,
    Into,
    PartialEq,
    Reflect,
    Serialize,
    Deserialize,
)]

pub struct DoorId(u32);

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
pub enum DoorStatus {
    #[default]
    Close,
    Open,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum OpenMethod {
    Click,
    Time,
    Item,
    Skill,
    Cycle,
}

#[derive(Clone, Debug, Default, Deref, Deserialize, Eq, PartialEq, Reflect, Serialize)]
pub struct OpenMethods(HashSet<OpenMethod>);

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[serde(default)]
pub struct DoorKind {
    pub id: DoorId,
    pub open_methods: OpenMethods,
    pub level: Option<u32>,
    pub hp_max: u32,
    pub hp_showable: bool,
    pub p_def: u32,
    pub m_def: Option<u32>,
    pub emitter_id: Option<u32>,
    pub status: DoorStatus,
    pub targetable: bool,
    pub check_collision: bool,
    pub hidden: bool,
}
