use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[repr(u32)]
#[derive(Clone, Component, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
pub enum MeshInfo {
    #[default]
    Default,
    Alternative,
    Alternative2,
}
