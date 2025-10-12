use crate::items::BodyPart;
use bevy::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub struct Armor {
    kind: ArmorKind,
    slot: Slot,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(i32)]
pub enum ArmorKind {
    Common(Slot),
    Sealed(Slot),
    Light(Slot),
    Heavy(Slot),
    Magic(Slot),
    Sigil,
    Shield,
    Underwear,
    Bracelet(BraceletSlot),
    Belt,
    Cloak,
    Accessory(AccessorySlot),
}

impl std::fmt::Display for ArmorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ArmorKind::Common(slot) => format!("Common {}", slot),
            ArmorKind::Sealed(slot) => format!("Sealed {}", slot),
            ArmorKind::Light(slot) => format!("Light {}", slot),
            ArmorKind::Heavy(slot) => format!("Heavy {}", slot),
            ArmorKind::Magic(slot) => format!("Magic {}", slot),
            ArmorKind::Sigil => "Sigil".to_string(),
            ArmorKind::Shield => "Shield".to_string(),
            ArmorKind::Underwear => "Underwear".to_string(),
            ArmorKind::Bracelet(bracelet_slot) => format!("{} Bracelet", bracelet_slot),
            ArmorKind::Belt => "Belt".to_string(),
            ArmorKind::Cloak => "Cloak".to_string(),
            ArmorKind::Accessory(accessory_slot) => format!("{} Accessory", accessory_slot),
        };
        write!(f, "{}", string)
    }
}

impl From<ArmorKind> for u32 {
    fn from(value: ArmorKind) -> Self {
        match value {
            ArmorKind::Common(slot) => 100 + u32::from(slot),
            ArmorKind::Sealed(slot) => 200 + u32::from(slot),
            ArmorKind::Light(slot) => 300 + u32::from(slot),
            ArmorKind::Heavy(slot) => 400 + u32::from(slot),
            ArmorKind::Magic(slot) => 500 + u32::from(slot),
            ArmorKind::Sigil => 600,
            ArmorKind::Shield => 601,
            ArmorKind::Underwear => 602,
            ArmorKind::Bracelet(bracelet_slot) => 700 + u32::from(bracelet_slot),
            ArmorKind::Belt => 800,
            ArmorKind::Cloak => 801,
            ArmorKind::Accessory(accessory_slot) => 900 + u32::from(accessory_slot),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    IntoPrimitive,
    EnumIter,
)]
#[repr(u8)]
pub enum Slot {
    Head,
    Chest,
    Legs,
    FullBody,
    Gloves,
    Feet,
}

impl From<Slot> for u32 {
    fn from(value: Slot) -> Self {
        value as u32
    }
}

impl From<u32> for Slot {
    fn from(value: u32) -> Self {
        match value {
            0 => Slot::Head,
            1 => Slot::Chest,
            2 => Slot::Legs,
            3 => Slot::FullBody,
            4 => Slot::Gloves,
            5 => Slot::Feet,
            _ => panic!("Invalid slot id: {}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum BraceletSlot {
    Left,
    Right,
}

impl From<BraceletSlot> for u32 {
    fn from(value: BraceletSlot) -> Self {
        value as u32
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum AccessorySlot {
    Left,
    Right,
    Both,
}

impl From<AccessorySlot> for u32 {
    fn from(value: AccessorySlot) -> Self {
        value as u32
    }
}

impl From<ArmorKind> for BodyPart {
    fn from(armor_type: ArmorKind) -> Self {
        match armor_type {
            ArmorKind::Common(slot)
            | ArmorKind::Sealed(slot)
            | ArmorKind::Light(slot)
            | ArmorKind::Heavy(slot)
            | ArmorKind::Magic(slot) => match slot {
                Slot::Head => BodyPart::Head,
                Slot::Chest => BodyPart::Chest,
                Slot::Legs => BodyPart::Legs,
                Slot::FullBody => BodyPart::FullBody,
                Slot::Gloves => BodyPart::Gloves,
                Slot::Feet => BodyPart::Feet,
            },
            ArmorKind::Sigil => BodyPart::LeftHand,
            ArmorKind::Shield => BodyPart::LeftHand,
            ArmorKind::Underwear => BodyPart::Underwear,
            ArmorKind::Bracelet(bracelet_slot) => match bracelet_slot {
                BraceletSlot::Left => BodyPart::LeftBracelet,
                BraceletSlot::Right => BodyPart::RightBracelet,
            },
            ArmorKind::Belt => BodyPart::Belt,
            ArmorKind::Cloak => BodyPart::Back,
            ArmorKind::Accessory(accessory_slot) => match accessory_slot {
                AccessorySlot::Left => BodyPart::AccessoryLeft,
                AccessorySlot::Right => BodyPart::AccessoryRight,
                AccessorySlot::Both => BodyPart::AccessoryBoth,
            },
        }
    }
}

impl From<u32> for ArmorKind {
    fn from(value: u32) -> Self {
        match value {
            100..=105 => ArmorKind::Common(Slot::from(value - 100)),
            200..=205 => ArmorKind::Sealed(Slot::from(value - 200)),
            300..=305 => ArmorKind::Light(Slot::from(value - 300)),
            400..=405 => ArmorKind::Heavy(Slot::from(value - 400)),
            500..=505 => ArmorKind::Magic(Slot::from(value - 500)),
            600 => ArmorKind::Sigil,
            601 => ArmorKind::Shield,
            602 => ArmorKind::Underwear,
            700 => ArmorKind::Bracelet(BraceletSlot::Left),
            701 => ArmorKind::Bracelet(BraceletSlot::Right),
            800 => ArmorKind::Belt,
            801 => ArmorKind::Cloak,
            900 => ArmorKind::Accessory(AccessorySlot::Left),
            901 => ArmorKind::Accessory(AccessorySlot::Right),
            902 => ArmorKind::Accessory(AccessorySlot::Both),
            _ => panic!("Invalid armor type id: {}", value),
        }
    }
}
