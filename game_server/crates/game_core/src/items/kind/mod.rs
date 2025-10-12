use bevy::prelude::*;
use l2r_core::utils::deserialize_array_20;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

mod armor;
mod consumable;
mod enchanting;
mod etc;
mod jewelry;
mod manor;
mod material;
mod pet;
mod recipe;
mod weapon;

pub use armor::*;
pub use consumable::*;
pub use enchanting::*;
pub use etc::*;
pub use jewelry::*;
pub use manor::*;
pub use material::*;
pub use pet::*;
pub use recipe::*;
pub use weapon::*;

#[derive(
    Clone, Copy, Debug, Deserialize, EnumDiscriminants, Event, PartialEq, Reflect, Serialize,
)]
#[strum_discriminants(name(KindVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter, Hash, Reflect, Serialize))]
pub enum Kind {
    Weapon(Weapon),
    Armor(ArmorKind),
    Jewelry(JewelryKind),
    Consumable(ConsumableKind),
    Material(MaterialType),
    Enchanting(EnchantingKind),
    Recipe(RecipeKind),
    Pet(PetItemKind),
    Manor(ManorItemKind),
    #[serde(deserialize_with = "deserialize_array_20")]
    Package([Package; 20]),
    Etc(EtcKind),
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Weapon(weapon) => write!(f, "Weapon: {}", weapon.kind),
            Kind::Armor(armor) => write!(f, "Armor: {}", armor),
            Kind::Jewelry(jewelry) => write!(f, "Jewelry: {}", jewelry),
            Kind::Consumable(consumable) => write!(f, "Consumable: {}", consumable),
            Kind::Material(material) => write!(f, "Material: {}", material),
            Kind::Enchanting(enchanting) => write!(f, "Enchanting: {}", enchanting),
            Kind::Recipe(recipe) => write!(f, "Recipe: {}", recipe),
            Kind::Pet(pet) => write!(f, "Pet: {}", pet),
            Kind::Manor(manor) => write!(f, "Manor: {}", manor),
            Kind::Package(_) => write!(f, "Package"),
            Kind::Etc(etc) => write!(f, "Etc: {}", etc),
        }
    }
}

impl Kind {
    pub fn shield(&self) -> bool {
        matches!(self, Kind::Armor(ArmorKind::Shield))
    }

    pub fn category_name(&self) -> String {
        match self {
            Kind::Weapon(weapon) => format!("Weapon: {}", weapon.kind),
            Kind::Armor(armor) => format!("Armor: {}", armor),
            Kind::Jewelry(jewelry) => format!("Jewelry: {}", jewelry),
            Kind::Consumable(_) => "Consumable".to_string(),
            Kind::Material(_) => "Material".to_string(),
            Kind::Enchanting(_) => "Enchanting".to_string(),
            Kind::Recipe(recipe) => format!("Recipe: {}", recipe),
            Kind::Pet(_) => "Pet".to_string(),
            Kind::Manor(manor) => format!("Manor: {}", manor),
            Kind::Package(_) => "Package".to_string(),
            Kind::Etc(_) => "Etc".to_string(),
        }
    }

    pub fn bow_or_crossbow(&self) -> bool {
        match self {
            Kind::Weapon(weapon) => {
                matches!(weapon.kind, WeaponKind::Bow | WeaponKind::Crossbow)
            }
            _ => false,
        }
    }
}

impl super::UsableItem for Kind {
    fn usable(&self) -> bool {
        match self {
            Kind::Consumable(consumable) => consumable.usable(),
            Kind::Enchanting(enchanting) => enchanting.usable(),
            Kind::Recipe(recipe) => recipe.usable(),
            _ => false,
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Etc(etc::EtcKind::default())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct Package {
    pub item_id: super::Id,
    pub min: u32,
    pub max: u32,
    pub chance: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, Event, IntoPrimitive, PartialEq, Reflect)]
#[repr(u16)]
pub enum SortingKind {
    Weapon,
    Armor,
    Jewelry,
    QestItem,
    Coins,
    Item,
}
impl SortingKind {
    pub fn to_le_bytes(self) -> [u8; 2] {
        (self as u16).to_le_bytes()
    }
}
impl From<&Kind> for SortingKind {
    fn from(kind: &Kind) -> Self {
        use Kind::*;
        match kind {
            Weapon(_) => SortingKind::Weapon,
            Armor(_) => SortingKind::Armor,
            Jewelry(_) => SortingKind::Jewelry,
            Etc(EtcKind::Quest) => SortingKind::QestItem,
            Etc(EtcKind::Coin) => SortingKind::Coins,
            _ => SortingKind::Item,
        }
    }
}

impl TryFrom<&Kind> for super::BodyPart {
    type Error = &'static str;
    fn try_from(kind: &Kind) -> Result<Self, Self::Error> {
        use Kind::*;
        match kind {
            Weapon(weapon) => Ok(weapon.kind.into()),
            Armor(armor_type) => Ok((*armor_type).into()),
            Jewelry(jewelry_type) => Ok((*jewelry_type).into()),
            _ => Err("This item don't have body part"),
        }
    }
}
