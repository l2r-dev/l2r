use crate::items::UsableItem;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum EnchantingKind {
    Scroll(ScrollTarget),
    LifeStone(LifeStoneType),
    SoulCrystal,
    EncantStone,
    Attribute,
}

impl From<EnchantingKind> for u32 {
    fn from(value: EnchantingKind) -> Self {
        match value {
            EnchantingKind::Scroll(_) => 0,
            EnchantingKind::LifeStone(_) => 1,
            EnchantingKind::SoulCrystal => 2,
            EnchantingKind::EncantStone => 3,
            EnchantingKind::Attribute => 4,
        }
    }
}

impl UsableItem for EnchantingKind {
    fn usable(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum LifeStoneType {
    Weapon(LifeStoneGrade),
    Accessory,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum LifeStoneGrade {
    None,
    Mid,
    High,
    Top,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum ScrollType {
    Common,
    Crystal,
    Blessed,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum ScrollTarget {
    Weapon(ScrollType),
    Armor(ScrollType),
}
