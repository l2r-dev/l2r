use crate::{
    items::{BodyPart, item_info::ItemSkill},
    stats::*,
};
use bevy::prelude::*;
use l2r_core::utils::deserialize_array_10;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use strum::Display;

#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
pub struct Weapon {
    #[serde(flatten)]
    pub kind: WeaponKind,
    #[serde(default)]
    pub random_damage: u32,
    #[serde(default)]
    pub range: PAtkRange,
    #[serde(default)]
    pub width: PAtkWidth,
    #[serde(default)]
    pub magical: bool,
    #[serde(default)]
    pub soulshots: u32,
    #[serde(default)]
    pub spiritshots: u32,
    // Kamael weapon exchange
    pub change_weapon_id: Option<crate::items::Id>,
    pub mp_consume: Option<u32>,
    pub reuse_delay: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_array_10")]
    pub enchanted_skill: [ItemSkill; 10],
    pub oncrit_chance: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_array_10")]
    pub oncrit_skill: [ItemSkill; 10],
    pub onmagic_chance: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_array_10")]
    pub onmagic_skill: [ItemSkill; 10],
}

impl PartialEq for Weapon {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Hash for Weapon {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
#[repr(u8)]
pub enum WeaponKind {
    Sword(SwordType),
    Blunt(BluntType),
    Bow,
    Crossbow,
    Dagger(DaggerType),
    Fist(FistType),
    Etc,
    FortFlag,
    FishingRod,
    Pole,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct WeaponAttackParams {
    pub is_bow: bool,
    pub reuse_delay: Option<u32>,
    pub primary_attack_delay_multiplier: f32,
    pub secondary_attack_delay_multiplier: Option<f32>,
}

impl Default for WeaponAttackParams {
    fn default() -> Self {
        WeaponAttackParams {
            primary_attack_delay_multiplier: 0.5,
            secondary_attack_delay_multiplier: None,
            reuse_delay: None,
            is_bow: false,
        }
    }
}

impl Weapon {
    pub fn attack_params(&self) -> WeaponAttackParams {
        WeaponAttackParams {
            reuse_delay: self.reuse_delay,
            ..self.kind.attack_params()
        }
    }
}

impl WeaponKind {
    fn attack_params(&self) -> WeaponAttackParams {
        let (primary_attack_delay_multiplier, secondary_attack_delay_multiplier) = match self {
            WeaponKind::Sword(sword) => match sword {
                SwordType::Ancient => (0.6, None),
                SwordType::Rapier => (0.4, None),
                SwordType::OneHanded => (0.55, None),
                SwordType::TwoHanded => (0.6, None),
                SwordType::Dual => (0.3, Some(0.35)),
            },
            WeaponKind::Blunt(blunt) => match blunt {
                BluntType::OneHanded => (0.55, None),
                BluntType::TwoHanded => (0.6, None),
                BluntType::Dual => (0.3, Some(0.35)),
            },
            WeaponKind::Dagger(dagger) => {
                if matches!(dagger, DaggerType::Dual) {
                    (0.3, Some(0.35))
                } else {
                    (0.55, None)
                }
            }
            WeaponKind::Fist(_) => (0.3, Some(0.35)),
            WeaponKind::Pole => (0.6, None),
            WeaponKind::Bow => (1.0, None),
            WeaponKind::Crossbow => (0.8, None),
            WeaponKind::Etc | WeaponKind::FortFlag | WeaponKind::FishingRod => (0.6, None),
        };
        WeaponAttackParams {
            primary_attack_delay_multiplier,
            secondary_attack_delay_multiplier,
            is_bow: matches!(self, WeaponKind::Bow | WeaponKind::Crossbow),
            reuse_delay: None,
        }
    }

    pub fn all_kinds() -> [super::Kind; 9] {
        [
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Sword(SwordType::OneHanded),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Sword(SwordType::TwoHanded),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Sword(SwordType::Dual),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Blunt(BluntType::OneHanded),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Blunt(BluntType::TwoHanded),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Bow,
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Crossbow,
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Dagger(DaggerType::OneHanded),
                ..Default::default()
            })),
            (super::Kind::Weapon(Weapon {
                kind: WeaponKind::Fist(FistType::OneHanded),
                ..Default::default()
            })),
        ]
    }
}

impl Default for WeaponKind {
    fn default() -> Self {
        WeaponKind::Fist(FistType::default())
    }
}

impl std::fmt::Display for WeaponKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let weapon_name = match self {
            WeaponKind::Sword(sword_type) => format!("{} Sword", sword_type),
            WeaponKind::Blunt(blunt_type) => format!("{} Blunt", blunt_type),
            WeaponKind::Bow => "Bow".to_string(),
            WeaponKind::Crossbow => "Crossbow".to_string(),
            WeaponKind::Dagger(dagger_type) => format!("{} Dagger", dagger_type),
            WeaponKind::Fist(fist_type) => format!("{} Fist", fist_type),
            WeaponKind::Etc => "Etc Weapon".to_string(),
            WeaponKind::FortFlag => "Fort Flag".to_string(),
            WeaponKind::FishingRod => "Fishing Rod".to_string(),
            WeaponKind::Pole => "Pole".to_string(),
        };
        write!(f, "{}", weapon_name)
    }
}

impl From<WeaponKind> for u32 {
    fn from(value: WeaponKind) -> Self {
        match value {
            WeaponKind::Sword(sword_type) => match sword_type {
                SwordType::Ancient => 100,
                SwordType::Rapier => 101,
                SwordType::OneHanded => 102,
                SwordType::TwoHanded => 103,
                SwordType::Dual => 104,
            },
            WeaponKind::Blunt(blunt_type) => match blunt_type {
                BluntType::OneHanded => 200,
                BluntType::TwoHanded => 201,
                BluntType::Dual => 202,
            },
            WeaponKind::Bow => 300,
            WeaponKind::Crossbow => 301,
            WeaponKind::Dagger(dagger_type) => match dagger_type {
                DaggerType::OneHanded => 400,
                DaggerType::Dual => 401,
            },
            WeaponKind::Fist(fist_type) => match fist_type {
                FistType::OneHanded => 500,
                FistType::Dual => 501,
            },
            WeaponKind::Etc => 600,
            WeaponKind::FortFlag => 601,
            WeaponKind::FishingRod => 602,
            WeaponKind::Pole => 603,
        }
    }
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum SwordType {
    Ancient,
    Rapier,
    #[default]
    OneHanded,
    TwoHanded,
    Dual,
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum BluntType {
    #[default]
    OneHanded,
    TwoHanded,
    Dual,
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum DaggerType {
    #[default]
    OneHanded,
    Dual,
}

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum FistType {
    #[default]
    OneHanded,
    Dual,
}

impl From<WeaponKind> for BodyPart {
    fn from(weapon_type: WeaponKind) -> Self {
        match weapon_type {
            WeaponKind::Sword(sword_type) => match sword_type {
                SwordType::OneHanded => BodyPart::RightHand,
                SwordType::Rapier => BodyPart::RightHand,
                _ => BodyPart::BothHand,
            },
            WeaponKind::Blunt(blunt_type) => match blunt_type {
                BluntType::OneHanded => BodyPart::RightHand,
                _ => BodyPart::BothHand,
            },
            WeaponKind::Bow => BodyPart::BothHand,
            WeaponKind::Crossbow => BodyPart::BothHand,
            WeaponKind::Dagger(dagger_type) => match dagger_type {
                DaggerType::OneHanded => BodyPart::RightHand,
                DaggerType::Dual => BodyPart::BothHand,
            },
            WeaponKind::Fist(fist_type) => match fist_type {
                FistType::OneHanded => BodyPart::RightHand,
                FistType::Dual => BodyPart::BothHand,
            },
            WeaponKind::Etc => BodyPart::RightHand,
            WeaponKind::FortFlag => BodyPart::BothHand,
            WeaponKind::FishingRod => BodyPart::RightHand,
            WeaponKind::Pole => BodyPart::BothHand,
        }
    }
}
