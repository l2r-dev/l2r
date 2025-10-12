use bevy::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Clone, Default, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize,
)]
#[repr(u8)]
pub enum ItemMaterial {
    Adamantaite,
    BloodSteel,
    Bone,
    Bronze,
    Chrysolite,
    Cloth,
    Cobweb,
    Cotton,
    Crystal,
    Damascus,
    Dyestuff,
    FineSteel,
    Fish,
    Gold,
    Horn,
    Leather,
    Liquid,
    Mithril,
    Oriharukon,
    Paper,
    Stone,
    ScaleOfDragon,
    Seed,
    Silver,
    #[default]
    Steel,
    Wood,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize, IntoPrimitive,
)]
#[repr(u8)]
pub enum MaterialType {
    Crystal,
    Common,
    Rare,
}

impl From<MaterialType> for u32 {
    fn from(value: MaterialType) -> Self {
        value as u32
    }
}
