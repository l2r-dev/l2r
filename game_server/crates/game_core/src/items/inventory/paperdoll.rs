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
    pub const fn base_p_def_slots() -> [DollSlot; 7] {
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
    pub const fn base_m_def_slots() -> [DollSlot; 5] {
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

    pub const fn bodypart_slots(bodypart: BodyPart) -> &'static [DollSlot] {
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
            BodyPart::BothHand => &[DollSlot::RightHand, DollSlot::LeftHand],
            BodyPart::FullBody => &[DollSlot::Chest, DollSlot::Legs],
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

    pub const USER_INFO_COUNT: usize = 26;
    pub const USER_INFO_COUNT_ERR: &'static str = "DollSlot::USER_INFO_COUNT should be updated in sync with the actual number of user info slots";

    pub const fn user_info_slots() -> [DollSlot; Self::USER_INFO_COUNT] {
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
    }

    pub fn char_info_slots() -> [DollSlot; 21] {
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
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        (*self as u32).to_le_bytes()
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PaperDoll([Option<ObjectId>; 25]);

#[derive(Clone, Copy, Debug)]
pub struct SlotItem {
    pub slot: DollSlot,
    pub object_id: Option<ObjectId>,
}

impl PaperDoll {
    pub fn get(&self, slot: DollSlot) -> Option<ObjectId> {
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

    pub fn slot_by_bodypart(&self, body_part: BodyPart) -> DollSlot {
        let slots = DollSlot::bodypart_slots(body_part);

        match body_part {
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
        }
    }

    pub fn unequip(&mut self, object_id: ObjectId) -> Option<DollSlot> {
        for slot in DollSlot::iter() {
            if self[slot] == Some(object_id) {
                self.unequip_slot(slot);
                return Some(slot);
            }
        }
        None
    }

    pub fn unequip_slot(&mut self, slot: DollSlot) -> Option<ObjectId> {
        let previous_item = self[slot];
        self[slot] = None;
        previous_item
    }

    pub fn equip_slot(&mut self, slot: DollSlot, object_id: ObjectId) {
        self[slot] = Some(object_id);
    }

    /// Returns the primary slot to equip and all slots that need to be unequipped
    pub fn desired_slot(
        &self,
        object_id: ObjectId,
        items_data: &impl ItemsDataAccess,
    ) -> Option<(DollSlot, Vec<DollSlot>)> {
        let item_id = items_data.item_by_object_id(object_id).ok()?.id();
        let item_info = items_data.item_info(item_id).ok()?;
        let body_part = item_info.bodypart()?;
        let mut unequip_slots = Vec::with_capacity(2);

        let desired_slot = match body_part {
            BodyPart::LeftHand => {
                if !item_info.kind().ammo()
                    && let Some(rh_oid) = self.get(DollSlot::RightHand)
                {
                    let rh_info = items_data.info_by_object_id(rh_oid).ok()?;
                    if rh_info.bodypart() == Some(BodyPart::BothHand) {
                        unequip_slots.push(DollSlot::RightHand);
                    }
                }
                self.slot_by_bodypart(body_part)
            }
            BodyPart::Legs => {
                if let Some(chest_oid) = self.get(DollSlot::Chest) {
                    let chest_info = items_data.info_by_object_id(chest_oid).ok()?;
                    if chest_info.bodypart() == Some(BodyPart::FullBody) {
                        unequip_slots.push(DollSlot::Chest);
                    }
                }
                self.slot_by_bodypart(body_part)
            }
            BodyPart::BothEar | BodyPart::BothFinger | BodyPart::Talisman => {
                let desired_slot = self.slot_by_bodypart(body_part);
                self.get(desired_slot)
                    .is_some()
                    .then(|| unequip_slots.push(desired_slot));
                desired_slot
            }
            _ => {
                unequip_slots.extend(
                    DollSlot::bodypart_slots(body_part)
                        .iter()
                        .filter(|&&slot| self.get(slot).is_some()),
                );
                self.slot_by_bodypart(body_part)
            }
        };

        Some((desired_slot, unequip_slots))
    }

    pub fn is_equipped(&self, object_id: ObjectId) -> bool {
        self.0.contains(&Some(object_id))
    }

    pub fn iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(slot, item)| (DollSlot::try_from(slot as u32).unwrap(), item))
            .filter_map(|(slot, item)| {
                item.map(|oid| SlotItem {
                    slot,
                    object_id: Some(oid),
                })
            })
    }

    pub fn user_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::user_info_slots()
            .into_iter()
            .map(|slot| SlotItem {
                slot,
                object_id: self[slot],
            })
    }

    pub fn char_info_iter(&self) -> impl Iterator<Item = SlotItem> + '_ {
        DollSlot::char_info_slots()
            .into_iter()
            .map(|slot| SlotItem {
                slot,
                object_id: self[slot],
            })
    }

    /// Checks if the given ammo item is compatible with the weapon in the right hand
    pub fn is_ammo_valid_for_weapon(
        &self,
        item_object_id: ObjectId,
        items_data: &impl ItemsDataAccess,
    ) -> bool {
        let Ok(ammo_info) = items_data.info_by_object_id(item_object_id) else {
            return false;
        };

        self.get(DollSlot::RightHand)
            .and_then(|rh_oid| items_data.info_by_object_id(rh_oid).ok())
            .map(|rh_info| rh_info.ammo_matches(ammo_info))
            .unwrap_or(false)
    }
}

impl From<Vec<crate::items::model::Model>> for PaperDoll {
    fn from(models: Vec<crate::items::model::Model>) -> Self {
        let mut paper_doll = PaperDoll::default();
        for model in models {
            if model.location == crate::items::ItemLocationVariant::PaperDoll
                && let Ok(slot) = DollSlot::try_from(model.location_data as u32)
            {
                paper_doll[slot] = Some(model.object_id);
            }
        }
        paper_doll
    }
}

impl Index<DollSlot> for PaperDoll {
    type Output = Option<ObjectId>;

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
            if let Some(object_id) = self.get(slot) {
                writeln!(f, "{slot:?}: ObjectId {object_id}")?;
            }
        }
        Ok(())
    }
}
