use crate::items::*;

#[derive(Clone, Copy, Debug, Event)]
pub struct AddInInventory {
    pub item: Entity,
    pub silent: bool, // If true, no system messages will be sent
}

impl AddInInventory {
    pub fn new(item: Entity) -> Self {
        Self {
            item,
            silent: false,
        }
    }

    pub fn new_silent(item: Entity) -> Self {
        Self { item, silent: true }
    }
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ItemObtained {
    pub item: Id,
    pub count: u64,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct DestroyItemRequest {
    pub item_oid: ObjectId,
    pub count: u64,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct DropIfPossible {
    pub item_oid: ObjectId,
    pub count: u64,
    pub location: Vec3,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct EquipItem(pub ObjectId);

#[derive(Clone, Copy, Debug, Event)]
pub struct ItemEquipped {
    pub item_object_id: ObjectId,
    pub slot: DollSlot,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ItemEquippedMessage(pub Item);

#[derive(Clone, Copy, Debug, Event)]
pub struct UnequipItem {
    pub item_object_id: ObjectId,
    pub skip_db_update: bool,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ItemUnequipped {
    pub item_object_id: ObjectId,
    pub slot: DollSlot,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ItemUnequippedMessage(pub Item);
