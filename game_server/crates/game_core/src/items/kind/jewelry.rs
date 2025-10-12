use crate::items::BodyPart;
use bevy::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize, IntoPrimitive,
)]
#[repr(u8)]
pub enum JewelryKind {
    Necklace,
    Earring,
    Ring,
}

impl From<JewelryKind> for u32 {
    fn from(value: JewelryKind) -> Self {
        value as u32
    }
}

impl From<JewelryKind> for BodyPart {
    fn from(jewelry_type: JewelryKind) -> Self {
        match jewelry_type {
            JewelryKind::Necklace => BodyPart::Neck,
            JewelryKind::Earring => BodyPart::BothEar,
            JewelryKind::Ring => BodyPart::BothFinger,
        }
    }
}
