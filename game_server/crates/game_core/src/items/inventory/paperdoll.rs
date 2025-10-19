use crate::items::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Deserialize;
use std::{
    fmt,
    ops::{Index, IndexMut},
};
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
    Talisman1,
    Talisman2,
    Talisman3,
    Talisman4,
    Talisman5,
    Talisman6,
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

    pub fn bodypart_slots(bodypart: BodyPart) -> &'static [DollSlot] {
        match bodypart {
            BodyPart::None => &[],
            BodyPart::Underwear => &[DollSlot::Underwear],
            BodyPart::RightEar => &[DollSlot::RightEar],
            BodyPart::LeftEar => &[DollSlot::LeftEar],
            BodyPart::BothEar => &[DollSlot::RightEar, DollSlot::LeftEar],
            BodyPart::Neck => &[DollSlot::Neck],
            BodyPart::RightFinger => &[DollSlot::RightFinger],
            BodyPart::LeftFinger => &[DollSlot::LeftFinger],
            BodyPart::BothFinger => &[DollSlot::RightFinger, DollSlot::LeftFinger],
            BodyPart::Head => &[DollSlot::Head],
            BodyPart::RightHand => &[DollSlot::RightHand],
            BodyPart::LeftHand => &[DollSlot::LeftHand],
            BodyPart::Gloves => &[DollSlot::Gloves],
            BodyPart::Chest => &[DollSlot::Chest],
            BodyPart::Legs => &[DollSlot::Legs],
            BodyPart::Feet => &[DollSlot::Feet],
            BodyPart::Back => &[DollSlot::Cloak],
            BodyPart::BothHand => &[DollSlot::RightHand],
            BodyPart::FullBody => &[DollSlot::Chest],
            BodyPart::AccessoryLeft => &[DollSlot::AccessoryLeft],
            BodyPart::AccessoryRight => &[DollSlot::AccessoryRight],
            BodyPart::AccessoryBoth => &[DollSlot::AccessoryRight, DollSlot::AccessoryLeft],
            BodyPart::RightBracelet => &[DollSlot::RightBracelet],
            BodyPart::LeftBracelet => &[DollSlot::LeftBracelet],
            BodyPart::Belt => &[DollSlot::Belt],
            BodyPart::Talisman => &[
                DollSlot::Talisman1,
                DollSlot::Talisman2,
                DollSlot::Talisman3,
                DollSlot::Talisman4,
                DollSlot::Talisman5,
                DollSlot::Talisman6,
            ],
            BodyPart::AllDress => &[DollSlot::Chest, DollSlot::Legs],
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
            Talisman1,
            Talisman2,
            Talisman3,
            Talisman4,
            Talisman5,
            Talisman6,
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
            Talisman1,
            Talisman2,
            Talisman3,
            Talisman4,
            Talisman5,
            Talisman6,
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
pub struct PaperDoll([Option<UniqueItem>; 25]);

#[derive(Debug)]
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
        self[slot]
    }

    pub fn first_free_slot(&self, slots: &[DollSlot]) -> Option<DollSlot> {
        slots
            .iter()
            .find(|slot| self.get(**slot).is_none())
            .copied()
    }

    fn free_or_first_slot(&self, slots: &[DollSlot]) -> DollSlot {
        self.first_free_slot(slots).unwrap_or_else(|| {
            *slots
                .first()
                .expect("at least one slot should be presented")
        })
    }

    pub fn unequip(&mut self, object_id: ObjectId) {
        let slots_to_clear: Vec<_> = self
            .0
            .iter()
            .enumerate()
            .map(|(slot, item)| (DollSlot::try_from(slot as u32).unwrap(), item))
            .filter_map(|(slot, item)| {
                if let Some(item) = item
                    && item.object_id() == object_id
                {
                    Some(slot)
                } else {
                    None
                }
            })
            .collect();

        for slot in slots_to_clear {
            self[slot] = None;
        }
    }

    pub fn equip_without_validations(
        &mut self,
        slot: DollSlot,
        item: UniqueItem,
    ) -> Option<UniqueItem> {
        let previous_item = self[slot];

        self[slot] = Some(item);

        previous_item
    }

    pub fn equip(
        &mut self,
        body_part: BodyPart,
        item: Option<UniqueItem>,
        item_info: &ItemInfo,
        items_infos: (&Assets<ItemsInfo>, &ItemsDataTable),
    ) -> (DollSlot, Vec<Option<UniqueItem>>) {
        let mut previous = Vec::with_capacity(2);

        let slots = DollSlot::bodypart_slots(body_part);

        let selected_slot = match body_part {
            BodyPart::BothEar | BodyPart::BothFinger | BodyPart::Talisman => {
                self.free_or_first_slot(slots)
            }
            _ => {
                // Select first slot as the primary slot
                slots
                    .first()
                    .copied()
                    .expect("At least one slot must exist")
            }
        };

        previous.push(self[selected_slot]);
        self[selected_slot] = item;

        match body_part {
            BodyPart::BothHand => {
                if let Some(left_info) = items_infos.item_info_from_uniq(&self[DollSlot::LeftHand])
                    && !item_info.ammo_matches(left_info)
                {
                    previous.push(self[DollSlot::LeftHand]);
                    self[DollSlot::LeftHand] = None;
                }
            }
            BodyPart::LeftHand => {
                if let Some(right_info) =
                    items_infos.item_info_from_uniq(&self[DollSlot::RightHand])
                    && !right_info.ammo_matches(item_info)
                {
                    previous.push(self[DollSlot::RightHand]);
                    self[DollSlot::RightHand] = None;
                }
            }
            BodyPart::FullBody => {
                previous.push(self[DollSlot::Legs]);
                self[DollSlot::Legs] = None;
            }
            BodyPart::Legs => {
                if let Some(v) = self[DollSlot::Chest]
                    && let Some(v) = v.item().bodypart()
                    && v == BodyPart::FullBody
                {
                    previous.push(self[DollSlot::Chest]);
                    self[DollSlot::Chest] = None;
                }
            }
            _ => {}
        }

        (selected_slot, previous)
    }

    pub fn is_equipped(&self, object_id: ObjectId) -> bool {
        self.0
            .iter()
            .any(|item| item.is_some_and(|i| i.object_id() == object_id))
    }

    pub fn iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(slot, item)| (DollSlot::try_from(slot as u32).unwrap(), item))
            .filter_map(|(slot, item)| item.as_ref().map(|item| SlotItem(slot, Some(*item))))
    }

    pub fn user_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::user_info_iter().map(|slot| SlotItem(slot, self[slot]))
    }

    pub fn char_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::char_info_iter().map(move |slot| SlotItem(slot, self[slot]))
    }
}

impl Index<DollSlot> for PaperDoll {
    type Output = Option<UniqueItem>;

    fn index(&self, index: DollSlot) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<DollSlot> for PaperDoll {
    fn index_mut(&mut self, index: DollSlot) -> &mut Self::Output {
        &mut self.0[index as usize]
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
