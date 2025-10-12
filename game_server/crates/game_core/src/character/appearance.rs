use crate::stats::Gender;
use bevy::prelude::*;
use sea_orm::{self, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
    Reflect,
    FromJsonQueryResult,
)]
pub struct Appearance {
    pub face: u32,
    pub hair_style: u32,
    pub hair_color: u32,
    pub gender: Gender,
}
