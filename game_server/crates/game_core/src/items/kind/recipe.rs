use crate::items::UsableItem;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    EnumIter,
    Display,
    Copy,
    Event,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Reflect,
)]
#[repr(u8)]
pub enum RecipeKind {
    Dwarf(Recipe),
    Common(Recipe),
}

impl From<RecipeKind> for u32 {
    fn from(value: RecipeKind) -> Self {
        match value {
            RecipeKind::Dwarf(_) => 0,
            RecipeKind::Common(_) => 1,
        }
    }
}

impl UsableItem for RecipeKind {
    fn usable(&self) -> bool {
        match self {
            RecipeKind::Dwarf(recipe) => recipe.level == 0,
            RecipeKind::Common(recipe) => recipe.level != 0,
        }
    }
}

#[derive(
    Clone, Copy, Default, Debug, Deserialize, Eq, Hash, Deref, PartialEq, Reflect, Serialize,
)]
pub struct RecipeId(u32);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub level: u32,
}
