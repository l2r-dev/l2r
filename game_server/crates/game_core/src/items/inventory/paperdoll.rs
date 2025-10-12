use crate::items::*;
use bevy::platform::collections::HashMap;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[repr(u32)]
#[derive(
    Clone,
    Copy,
    Debug,
    EnumIter,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    IntoPrimitive,
    TryFromPrimitive,
    Deserialize,
)]
pub enum DollSlot {
    Underwear,
    Head,
    AccessoryLeft,
    AccessoryRight,
    Neck,
    RightHand,
    Chest,
    LeftHand,
    RightEar,
    LeftEar,
    Gloves,
    Legs,
    Feet,
    RightFinger,
    LeftFinger,
    LeftBracelet,
    RightBracelet,
    Deco1,
    Deco2,
    Deco3,
    Deco4,
    Deco5,
    Deco6,
    Cloak,
    Belt,
}

impl DollSlot {
    pub fn base_p_def_slots() -> [DollSlot; 7] {
        [
            DollSlot::Chest,
            DollSlot::Legs,
            DollSlot::Head,
            DollSlot::Feet,
            DollSlot::Gloves,
            DollSlot::Underwear,
            DollSlot::Cloak,
        ]
    }
    pub fn base_m_def_slots() -> [DollSlot; 5] {
        [
            DollSlot::RightEar,
            DollSlot::LeftEar,
            DollSlot::RightFinger,
            DollSlot::LeftFinger,
            DollSlot::Neck,
        ]
    }

    pub const fn base_p_def(&self) -> f32 {
        match self {
            DollSlot::Chest => 31.0,
            DollSlot::Legs => 18.0,
            DollSlot::Head => 12.0,
            DollSlot::Feet => 7.0,
            DollSlot::Gloves => 8.0,
            DollSlot::Underwear => 3.0,
            DollSlot::Cloak => 1.0,
            _ => 0.0,
        }
    }

    pub const fn base_m_def(&self) -> f32 {
        match self {
            DollSlot::RightEar => 9.0,
            DollSlot::LeftEar => 9.0,
            DollSlot::RightFinger => 5.0,
            DollSlot::LeftFinger => 5.0,
            DollSlot::Neck => 13.0,
            _ => 0.0,
        }
    }

    pub fn base_p_def_total() -> f32 {
        DollSlot::base_p_def_slots()
            .iter()
            .map(|&slot| slot.base_p_def())
            .sum()
    }

    pub fn base_m_def_total() -> f32 {
        DollSlot::base_m_def_slots()
            .iter()
            .map(|&slot| slot.base_m_def())
            .sum()
    }

    fn deco_slots() -> [DollSlot; 6] {
        [
            DollSlot::Deco1,
            DollSlot::Deco2,
            DollSlot::Deco3,
            DollSlot::Deco4,
            DollSlot::Deco5,
            DollSlot::Deco6,
        ]
    }
    pub fn bodypart_slots(bodypart: BodyPart) -> Vec<Option<DollSlot>> {
        match bodypart {
            BodyPart::None => vec![None],
            BodyPart::Underwear => vec![Some(DollSlot::Underwear)],
            BodyPart::RightEar => vec![Some(DollSlot::RightEar)],
            BodyPart::LeftEar => vec![Some(DollSlot::LeftEar)],
            BodyPart::BothEar => vec![Some(DollSlot::RightEar), Some(DollSlot::LeftEar)],
            BodyPart::Neck => vec![Some(DollSlot::Neck)],
            BodyPart::RightFinger => vec![Some(DollSlot::RightFinger)],
            BodyPart::LeftFinger => vec![Some(DollSlot::LeftFinger)],
            BodyPart::BothFinger => vec![Some(DollSlot::RightFinger), Some(DollSlot::LeftFinger)],
            BodyPart::Head => vec![Some(DollSlot::Head)],
            BodyPart::RightHand => vec![Some(DollSlot::RightHand)],
            BodyPart::LeftHand => vec![Some(DollSlot::LeftHand)],
            BodyPart::Gloves => vec![Some(DollSlot::Gloves)],
            BodyPart::Chest => vec![Some(DollSlot::Chest)],
            BodyPart::Legs => vec![Some(DollSlot::Legs)],
            BodyPart::Feet => vec![Some(DollSlot::Feet)],
            BodyPart::Back => vec![Some(DollSlot::Cloak)],
            BodyPart::BothHand => vec![Some(DollSlot::RightHand), Some(DollSlot::LeftHand)],
            BodyPart::FullBody => vec![Some(DollSlot::Chest), Some(DollSlot::Legs)],
            BodyPart::AccessoryLeft => vec![Some(DollSlot::AccessoryLeft)],
            BodyPart::AccessoryRight => vec![Some(DollSlot::AccessoryRight)],
            BodyPart::AccessoryBoth => vec![
                Some(DollSlot::AccessoryRight),
                Some(DollSlot::AccessoryLeft),
            ],
            BodyPart::RightBracelet => vec![Some(DollSlot::RightBracelet)],
            BodyPart::LeftBracelet => vec![Some(DollSlot::LeftBracelet)],
            BodyPart::Belt => vec![Some(DollSlot::Belt)],
            BodyPart::Deco => DollSlot::deco_slots()
                .iter()
                .map(|slot| Some(*slot))
                .collect(),
            BodyPart::AllDress => vec![Some(DollSlot::Chest), Some(DollSlot::Legs)],
        }
    }
    pub fn user_info_iter() -> impl Iterator<Item = DollSlot> {
        use DollSlot::*;
        [
            Underwear,
            RightEar,
            LeftEar,
            Neck,
            RightFinger,
            LeftFinger,
            Head,
            RightHand,
            LeftHand,
            Gloves,
            Chest,
            Legs,
            Feet,
            Cloak,
            RightHand,
            AccessoryLeft,
            AccessoryRight,
            RightBracelet,
            LeftBracelet,
            Deco1,
            Deco2,
            Deco3,
            Deco4,
            Deco5,
            Deco6,
            Belt,
        ]
        .iter()
        .copied()
    }
    pub fn char_info_iter() -> impl Iterator<Item = DollSlot> {
        use DollSlot::*;
        [
            Underwear,
            Head,
            RightHand,
            LeftHand,
            Gloves,
            Chest,
            Legs,
            Feet,
            Cloak,
            RightHand,
            AccessoryLeft,
            AccessoryRight,
            RightBracelet,
            LeftBracelet,
            Deco1,
            Deco2,
            Deco3,
            Deco4,
            Deco5,
            Deco6,
            Belt,
        ]
        .iter()
        .copied()
    }
    pub fn to_le_bytes(&self) -> [u8; 4] {
        (*self as u32).to_le_bytes()
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PaperDoll(HashMap<DollSlot, Option<UniqueItem>>);

pub struct SlotItem(pub DollSlot, pub Option<UniqueItem>);
impl SlotItem {
    pub fn slot(&self) -> DollSlot {
        self.0
    }

    pub fn unique_item(&self) -> Option<&UniqueItem> {
        self.1.as_ref()
    }
}

impl PaperDoll {
    pub fn get(&self, slot: DollSlot) -> Option<UniqueItem> {
        self.0.get(&slot).cloned().unwrap_or(None)
    }

    pub fn first_free_slot(&self, slots: &[DollSlot]) -> Option<DollSlot> {
        slots
            .iter()
            .find(|slot| self.get(**slot).is_none())
            .copied()
    }

    fn free_or_random_slot(&self, slots: &[DollSlot]) -> DollSlot {
        let mut rng = rand::thread_rng();
        self.first_free_slot(slots)
            .unwrap_or_else(|| *slots.choose(&mut rng).unwrap())
    }

    pub fn unequip(&mut self, object_id: ObjectId) {
        let slots_to_clear: Vec<_> = self
            .0
            .iter()
            .filter_map(|(slot, item)| {
                if let Some(item) = item {
                    if item.object_id() == object_id {
                        Some(*slot)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for slot in slots_to_clear {
            self.0.insert(slot, None);
        }
    }

    pub fn equip(
        &mut self,
        body_part: BodyPart,
        item: Option<UniqueItem>,
    ) -> (DollSlot, Vec<Option<UniqueItem>>) {
        let mut previous = Vec::with_capacity(2);
        let slots = DollSlot::bodypart_slots(body_part)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        match body_part {
            BodyPart::BothEar | BodyPart::BothFinger => {
                let selected_slot = self.free_or_random_slot(slots.as_slice());
                previous.push(self.0.get(&selected_slot).cloned().unwrap_or(None));
                self.0.insert(selected_slot, item);
                (selected_slot, previous)
            }
            _ => {
                // Select first slot as the primary slot
                let selected_slot = slots
                    .first()
                    .copied()
                    .expect("At least one slot must exist");
                for slot in slots {
                    previous.push(self.0.get(&slot).cloned().unwrap_or(None));
                    self.0.insert(slot, item);
                }
                (selected_slot, previous)
            }
        }
    }

    pub fn is_equipped(&self, object_id: ObjectId) -> bool {
        self.0
            .iter()
            .any(|(_, item)| item.is_some_and(|i| i.object_id() == object_id))
    }

    pub fn iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        self.0
            .iter()
            .filter_map(|(slot, item)| item.as_ref().map(|item| SlotItem(*slot, Some(*item))))
    }

    pub fn user_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::user_info_iter().map(move |slot| SlotItem(slot, self.get(slot)))
    }

    pub fn char_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::char_info_iter().map(move |slot| SlotItem(slot, self.get(slot)))
    }
}

impl fmt::Display for PaperDoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for slot in DollSlot::iter() {
            if let Some(item) = self.get(slot) {
                writeln!(f, "{slot:?}: Item {item}")?;
            }
        }
        Ok(())
    }
}
