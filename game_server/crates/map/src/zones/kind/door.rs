use bevy::{platform::collections::HashSet, prelude::*};
use bevy_enum_tag::EnumComponentTag;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

pub struct DoorsComponentsPlugin;
impl Plugin for DoorsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DoorId>()
            .register_type::<DoorStatus>()
            .register_type::<OpenMethod>()
            .register_type::<OpenMethods>()
            .register_type::<DoorKind>();
    }
}

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
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumComponentTag, Reflect, Serialize)]
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
