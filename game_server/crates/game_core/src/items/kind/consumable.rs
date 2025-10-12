use crate::items::UsableItem;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(i32)]
pub enum ConsumableKind {
    Ammo(AmmoKind),
    Herb,
    Potion,
    Elixir,
    Scroll,
    Shot(ShotKind),
}

impl std::fmt::Display for ConsumableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ConsumableKind::*;
        match self {
            Ammo(ammo) => write!(f, "Ammo: {}", ammo),
            Herb => write!(f, "Herb"),
            Potion => write!(f, "Potion"),
            Elixir => write!(f, "Elixir"),
            Scroll => write!(f, "Scroll"),
            Shot(shot) => write!(f, "Shot: {}", shot),
        }
    }
}

impl UsableItem for ConsumableKind {
    fn usable(&self) -> bool {
        use ConsumableKind::*;
        match self {
            Ammo(_) => false,
            Herb => true,
            Potion => true,
            Elixir => true,
            Scroll => true,
            Shot(_) => true,
        }
    }
}

impl From<ConsumableKind> for u32 {
    fn from(value: ConsumableKind) -> Self {
        use ConsumableKind::*;
        use ShotKind::*;
        match value {
            Ammo(ammo) => match ammo {
                AmmoKind::Arrow => 100,
                AmmoKind::Bolt => 101,
            },
            Herb => 200,
            Potion => 201,
            Elixir => 202,
            Scroll => 203,
            Shot(shot) => match shot {
                Soulshot => 300,
                Spiritshot => 301,
                BlessedSpiritshot => 302,
                Fishing => 303,
            },
        }
    }
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum AmmoKind {
    #[default]
    Arrow,
    Bolt,
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum ShotKind {
    #[default]
    Soulshot,
    Spiritshot,
    BlessedSpiritshot,
    Fishing,
}

#[derive(Clone, Component, Copy)]
#[component(storage = "SparseSet")]
pub struct Soulshot;

#[derive(Clone, Component, Copy)]
#[component(storage = "SparseSet")]
pub struct Spiritshot;

#[derive(Clone, Component, Copy)]
#[component(storage = "SparseSet")]
pub struct BlessedSpiritshot;

#[derive(Clone, Component, Copy)]
#[component(storage = "SparseSet")]
pub struct FishingShot;
