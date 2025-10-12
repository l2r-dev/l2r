use bevy::prelude::*;
use num_enum::IntoPrimitive;

mod destination;
mod id;

pub use destination::*;
pub use id::*;
use serde::Deserialize;
use strum::{Display, EnumIter, EnumString};

pub struct TeleportComponentsPlugin;
impl Plugin for TeleportComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TeleportInProgress>();
    }
}

#[derive(Component, Debug, Reflect)]
#[component(storage = "SparseSet")]
pub struct TeleportInProgress;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    EnumIter,
    EnumString,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    Reflect,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TeleportListKind {
    #[default]
    Common,
    Noble,
    Floor,
    Other,
}

#[derive(Clone, Copy, Debug, Default, IntoPrimitive, Reflect)]
#[repr(u32)]
pub enum TeleportType {
    #[default]
    FADE,
    INSTANT,
}
