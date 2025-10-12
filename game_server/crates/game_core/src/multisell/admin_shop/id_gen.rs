use crate::{
    items::{Grade, Kind},
    multisell::Id,
};
use bevy::prelude::*;

const ADMIN_SHOP_ID_START: u32 = 1_000_000_000;
const ADMIN_SHOP_GROUP_SPACING: u32 = 10_000;

const ADMIN_SHOP_WEAPON_OFFSET: u32 = ADMIN_SHOP_ID_START;
const ADMIN_SHOP_ARMOR_OFFSET: u32 = ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_JEWELRY_OFFSET: u32 = 2 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_CONSUMABLE_OFFSET: u32 = 3 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_MATERIAL_OFFSET: u32 = 4 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_ENCHANTING_OFFSET: u32 = 5 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_RECIPE_OFFSET: u32 = 6 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_PET_OFFSET: u32 = 7 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_MANOR_OFFSET: u32 = 8 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_PACKAGE_OFFSET: u32 = 9 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;
const ADMIN_SHOP_ETC_OFFSET: u32 = 10 * ADMIN_SHOP_GROUP_SPACING + ADMIN_SHOP_ID_START;

impl From<(Kind, Grade)> for Id {
    fn from((kind, grade): (Kind, Grade)) -> Self {
        use crate::items::kind::Kind::*;
        let base_id: u32 = match kind {
            Weapon(weapon) => ADMIN_SHOP_WEAPON_OFFSET + u32::from(weapon.kind),
            Armor(armor_type) => ADMIN_SHOP_ARMOR_OFFSET + u32::from(armor_type),
            Jewelry(jewelry_type) => ADMIN_SHOP_JEWELRY_OFFSET + u32::from(jewelry_type),
            Consumable(consumable_type) => {
                ADMIN_SHOP_CONSUMABLE_OFFSET + u32::from(consumable_type)
            }
            Material(material_type) => ADMIN_SHOP_MATERIAL_OFFSET + u32::from(material_type),
            Enchanting(enchanting_type) => {
                ADMIN_SHOP_ENCHANTING_OFFSET + u32::from(enchanting_type)
            }
            Recipe(recipe_type) => ADMIN_SHOP_RECIPE_OFFSET + u32::from(recipe_type),
            Pet(pet_type) => ADMIN_SHOP_PET_OFFSET + u32::from(pet_type),
            Manor(manor_type) => ADMIN_SHOP_MANOR_OFFSET + u32::from(manor_type),
            Package(_) => ADMIN_SHOP_PACKAGE_OFFSET,
            Etc(etc_type) => ADMIN_SHOP_ETC_OFFSET + u32::from(etc_type),
        };

        // For items that support grades (weapon, armor, jewelry), add grade offset
        let final_id = match kind {
            Kind::Weapon(_) | Kind::Armor(_) | Kind::Jewelry(_) => {
                base_id + (u32::from(grade) * 10)
            }
            // Other kinds don't use grades
            _ => base_id,
        };

        final_id.into()
    }
}

impl Id {
    pub fn to_kind_and_grade(self) -> Result<(Kind, Grade), String> {
        (self).try_into()
    }
}

impl TryFrom<Id> for (Kind, Grade) {
    type Error = String;

    fn try_from(id: Id) -> Result<Self, Self::Error> {
        use crate::items::kind::*;

        let id_val = *id;

        match id_val {
            x if (ADMIN_SHOP_WEAPON_OFFSET
                ..=ADMIN_SHOP_WEAPON_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Weapon - supports grades
                let relative_id = id_val - ADMIN_SHOP_WEAPON_OFFSET;
                // The relative_id contains: weapon_type_id + (grade * 10)
                // Grade can add 0, 10, 20, 30, 40, 50, 60, 70
                // Find which grade offset fits
                let mut weapon_id = 0;
                let mut grade_val = 0;

                for grade in 0..=7 {
                    let test_weapon_id = relative_id - (grade * 10);
                    // Check if this weapon_id is valid by trying to match it
                    if matches!(test_weapon_id, 100..=104 | 200..=202 | 300..=301 | 400..=401 | 500..=501 | 600..=603)
                    {
                        weapon_id = test_weapon_id;
                        grade_val = grade;
                        break;
                    }
                }

                let grade = Grade::from(grade_val);
                use WeaponKind::*;
                let weapon_kind = match weapon_id {
                    100 => Sword(SwordType::Ancient),
                    101 => Sword(SwordType::Rapier),
                    102 => Sword(SwordType::OneHanded),
                    103 => Sword(SwordType::TwoHanded),
                    104 => Sword(SwordType::Dual),
                    200 => Blunt(BluntType::OneHanded),
                    201 => Blunt(BluntType::TwoHanded),
                    202 => Blunt(BluntType::Dual),
                    300 => Bow,
                    301 => Crossbow,
                    400 => Dagger(DaggerType::OneHanded),
                    401 => Dagger(DaggerType::Dual),
                    500 => Fist(FistType::OneHanded),
                    501 => Fist(FistType::Dual),
                    600 => Etc,
                    601 => FortFlag,
                    602 => FishingRod,
                    603 => Pole,
                    _ => return Err(format!("Invalid weapon id: {}", weapon_id)),
                };

                let weapon = Weapon {
                    kind: weapon_kind,
                    ..Default::default()
                };
                Ok((Kind::Weapon(weapon), grade))
            }

            x if (ADMIN_SHOP_ARMOR_OFFSET
                ..=ADMIN_SHOP_ARMOR_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Armor - supports grades
                let relative_id = id_val - ADMIN_SHOP_ARMOR_OFFSET;

                let mut armor_id = 0;
                let mut grade_val = 0;

                for grade in 0..=7 {
                    let test_armor_id = relative_id - (grade * 10);
                    // Check if this armor_id is valid
                    if matches!(test_armor_id, 100..=105 | 200..=205 | 300..=305 | 400..=405 | 500..=505 | 600..=602 | 700..=701 | 800..=801 | 900..=902)
                    {
                        armor_id = test_armor_id;
                        grade_val = grade;
                        break;
                    }
                }

                let grade = Grade::from(grade_val);
                let armor_type = ArmorKind::from(armor_id);
                Ok((Kind::Armor(armor_type), grade))
            }

            x if (ADMIN_SHOP_JEWELRY_OFFSET
                ..=ADMIN_SHOP_JEWELRY_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Jewelry - supports grades
                let relative_id = id_val - ADMIN_SHOP_JEWELRY_OFFSET;

                let mut jewelry_id = 0;
                let mut grade_val = 0;

                for grade in 0..=7 {
                    let test_jewelry_id = relative_id - (grade * 10);
                    // Check if this jewelry_id is valid (0, 1, 2)
                    if matches!(test_jewelry_id, 0..=2) {
                        jewelry_id = test_jewelry_id;
                        grade_val = grade;
                        break;
                    }
                }

                let grade = Grade::from(grade_val);
                use JewelryKind::*;
                let jewelry_type = match jewelry_id {
                    0 => Necklace,
                    1 => Earring,
                    2 => Ring,
                    _ => return Err(format!("Invalid jewelry type: {}", jewelry_id)),
                };

                Ok((Kind::Jewelry(jewelry_type), grade))
            }

            x if (ADMIN_SHOP_CONSUMABLE_OFFSET
                ..=ADMIN_SHOP_CONSUMABLE_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Consumable - no grades
                let consumable_id = id_val - ADMIN_SHOP_CONSUMABLE_OFFSET;
                use ConsumableKind::*;
                let consumable_type = match consumable_id {
                    100 => Ammo(AmmoKind::Arrow),
                    101 => Ammo(AmmoKind::Bolt),
                    200 => Herb,
                    201 => Potion,
                    202 => Elixir,
                    203 => Scroll,
                    300 => Shot(ShotKind::Soulshot),
                    301 => Shot(ShotKind::Spiritshot),
                    302 => Shot(ShotKind::BlessedSpiritshot),
                    303 => Shot(ShotKind::Fishing),
                    _ => return Err(format!("Invalid consumable type: {}", consumable_id)),
                };
                Ok((Kind::Consumable(consumable_type), Grade::None))
            }

            x if (ADMIN_SHOP_MATERIAL_OFFSET
                ..=ADMIN_SHOP_MATERIAL_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Material - no grades
                let material_id = id_val - ADMIN_SHOP_MATERIAL_OFFSET;
                use MaterialType::*;
                let material_type = match material_id {
                    0 => Crystal,
                    1 => Common,
                    2 => Rare,
                    _ => return Err(format!("Invalid material type: {}", material_id)),
                };
                Ok((Kind::Material(material_type), Grade::None))
            }

            x if (ADMIN_SHOP_ENCHANTING_OFFSET
                ..=ADMIN_SHOP_ENCHANTING_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Enchanting - no grades
                let enchanting_id = id_val - ADMIN_SHOP_ENCHANTING_OFFSET;
                use EnchantingKind::*;
                let enchanting_type = match enchanting_id {
                    0 => Scroll(ScrollTarget::Weapon(ScrollType::Common)),
                    1 => LifeStone(LifeStoneType::Weapon(LifeStoneGrade::None)),
                    2 => SoulCrystal,
                    3 => EncantStone,
                    4 => Attribute,
                    _ => return Err(format!("Invalid enchanting type: {}", enchanting_id)),
                };
                Ok((Kind::Enchanting(enchanting_type), Grade::None))
            }

            x if (ADMIN_SHOP_RECIPE_OFFSET
                ..=ADMIN_SHOP_RECIPE_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Recipe - no grades
                let recipe_id = id_val - ADMIN_SHOP_RECIPE_OFFSET;
                let recipe_type = match recipe_id {
                    0 => RecipeKind::Dwarf(Recipe::default()), // Default recipe
                    1 => RecipeKind::Common(Recipe::default()), // Default recipe
                    _ => return Err(format!("Invalid recipe type: {}", recipe_id)),
                };
                Ok((Kind::Recipe(recipe_type), Grade::None))
            }

            x if (ADMIN_SHOP_PET_OFFSET..=ADMIN_SHOP_PET_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Pet - no grades
                let pet_id = id_val - ADMIN_SHOP_PET_OFFSET;
                use PetItemKind::*;
                let pet_type = match pet_id {
                    0 => Collar,
                    1 => Weapon,
                    2 => Armor,
                    3 => Shot(ShotKind::Soulshot), // Default shot type
                    4 => Consumable,
                    _ => return Err(format!("Invalid pet type: {}", pet_id)),
                };
                Ok((Kind::Pet(pet_type), Grade::None))
            }

            x if (ADMIN_SHOP_MANOR_OFFSET
                ..=ADMIN_SHOP_MANOR_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Manor - no grades
                let manor_id = id_val - ADMIN_SHOP_MANOR_OFFSET;
                let manor_type = match manor_id {
                    0 => ManorItemKind::Harvest,
                    1 => ManorItemKind::Crop,
                    2 => ManorItemKind::MatureCrop,
                    3 => ManorItemKind::Seed,
                    4 => ManorItemKind::AltSeed,
                    _ => return Err(format!("Invalid manor type: {}", manor_id)),
                };
                Ok((Kind::Manor(manor_type), Grade::None))
            }

            x if (ADMIN_SHOP_PACKAGE_OFFSET
                ..=ADMIN_SHOP_PACKAGE_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Package - special case, no grades
                Ok((Kind::Package([Package::default(); 20]), Grade::None))
            }

            x if (ADMIN_SHOP_ETC_OFFSET..=ADMIN_SHOP_ETC_OFFSET + ADMIN_SHOP_GROUP_SPACING - 1)
                .contains(&x) =>
            {
                // Etc - no grades
                let etc_id = id_val - ADMIN_SHOP_ETC_OFFSET;
                let etc_type = EtcKind::from(etc_id);
                Ok((Kind::Etc(etc_type), Grade::None))
            }

            _ => Err(format!("Invalid multisell ID: {}", id_val)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::kind::*;

    #[test]
    fn test_all_weapon_kinds_round_trip() {
        use crate::items::Grade;
        use strum::IntoEnumIterator;

        for weapon_kind in WeaponKind::all_kinds().iter() {
            for grade in Grade::iter() {
                let original = (*weapon_kind, grade);
                let id = Id::from(original);
                let (converted_kind, converted_grade) = id.to_kind_and_grade().unwrap();

                assert_eq!(converted_grade, grade);

                if let (Kind::Weapon(original_weapon), Kind::Weapon(converted_weapon)) =
                    (weapon_kind, &converted_kind)
                {
                    assert_eq!(original_weapon.kind, converted_weapon.kind);
                }
            }
        }
    }
}
