use bevy::prelude::*;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Reflect,
    Display,
    EnumIter,
    FromPrimitive,
)]
#[repr(u8)]
pub enum Grade {
    #[default]
    None,
    D,
    C,
    B,
    A,
    S,
    S80,
    S84,
}

impl Grade {
    pub fn shot_grade(&self) -> Self {
        match self {
            Self::S80 | Self::S84 => Self::S,
            _ => *self,
        }
    }
}

impl From<Grade> for u32 {
    fn from(value: Grade) -> Self {
        value as u32
    }
}

impl From<u32> for Grade {
    fn from(value: u32) -> Self {
        Grade::from_primitive(value as u8)
    }
}

// // 0 - armor, 1 - weapon
// #[derive(Clone, Debug)]
// pub struct CrystalEnchantBonus(HashMap<Grade, (u32, u32)>);
// impl CrystalEnchantBonus {
//     pub fn get(&self, grade: Grade) -> (u32, u32) {
//         self.0[&grade]
//     }
// }
// impl Default for CrystalEnchantBonus {
//     fn default() -> Self {
//         let mut map = HashMap::default();
//         map.insert(Grade::D, (11, 90));
//         map.insert(Grade::C, (6, 45));
//         map.insert(Grade::B, (11, 67));
//         map.insert(Grade::A, (20, 145));
//         map.insert(Grade::S, (25, 250));
//         map.insert(Grade::S80, (25, 250));
//         map.insert(Grade::S84, (25, 250));
//         Self(map)
//     }
// }
