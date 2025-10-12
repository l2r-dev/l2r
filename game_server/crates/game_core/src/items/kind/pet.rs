use super::consumable::ShotKind;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum PetItemKind {
    Collar,
    Weapon,
    Armor,
    Shot(ShotKind),
    Consumable,
}

impl From<PetItemKind> for u32 {
    fn from(value: PetItemKind) -> Self {
        use PetItemKind::*;
        match value {
            Collar => 0,
            Weapon => 1,
            Armor => 2,
            Shot(_) => 3,
            Consumable => 4,
        }
    }
}
