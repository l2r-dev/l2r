use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum Attribute {
    Fire,
    Water,
    Wind,
    Earth,
    Dark,
    Holy,
}
