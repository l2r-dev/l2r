use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum ManorItemKind {
    #[default]
    Harvest,
    Crop,
    MatureCrop,
    Seed,
    AltSeed,
}

impl From<ManorItemKind> for u32 {
    fn from(value: ManorItemKind) -> Self {
        value as u32
    }
}
