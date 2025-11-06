use crate::items::*;
use smallvec::SmallVec;

#[derive(Event)]
pub struct AddInInventory {
    pub items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>,
    pub silent: bool, // If true, no system messages will be sent
}

impl AddInInventory {
    pub fn new(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: false,
        }
    }

    pub fn new_silent(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: true,
        }
    }
}

#[derive(Clone, Debug, Event)]
pub struct AddNonStackable {
    pub items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>,
    pub silent: bool, // If true, no system messages will be sent
}

impl AddNonStackable {
    pub fn new(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: false,
        }
    }

    pub fn new_silent(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: true,
        }
    }
}

#[derive(Clone, Debug, Event)]
pub struct AddStackable {
    pub items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>,
    pub silent: bool, // If true, no system messages will be sent
}

impl AddStackable {
    pub fn new(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: false,
        }
    }

    pub fn new_silent(items: SmallVec<[Entity; ITEMS_OPERATION_STACK]>) -> Self {
        Self {
            items,
            silent: true,
        }
    }
}

#[derive(Event)]
pub struct DestroyItemRequest {
    pub item_oid: ObjectId,
    pub count: u64,
}

#[derive(Event)]
pub struct DropIfPossible {
    pub item_oid: ObjectId,
    pub count: u64,
    pub location: Vec3,
}

#[derive(Clone, Debug, Event)]
pub struct DropItemEvent {
    entity: Entity,
    pub item_oid: ObjectId,
    pub count: u64,
    pub location: Vec3,
}

impl ContainsEntity for DropItemEvent {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl DropItemEvent {
    pub fn new(entity: Entity, item_oid: ObjectId, count: u64, location: Vec3) -> Self {
        Self {
            entity,
            item_oid,
            count,
            location,
        }
    }
}

#[derive(Clone, Debug, Event)]
pub struct EquipItem {
    pub item_object_id: ObjectId,
}

impl EquipItem {
    pub fn new(item_object_id: ObjectId) -> Self {
        Self { item_object_id }
    }
}

#[derive(Clone, Debug, Event)]
pub struct ItemEquipped {
    pub item_object_id: ObjectId,
}

impl ItemEquipped {
    pub fn new(item_object_id: ObjectId) -> Self {
        Self { item_object_id }
    }
}

#[derive(Clone, Debug, Event)]
pub struct UnequipItem {
    pub item_object_id: ObjectId,
    pub skip_db_update: bool,
}

impl UnequipItem {
    pub fn new(item_object_id: ObjectId) -> Self {
        Self {
            item_object_id,
            skip_db_update: false,
        }
    }

    pub fn new_skip_db(item_object_id: ObjectId) -> Self {
        Self {
            item_object_id,
            skip_db_update: true,
        }
    }
}

#[derive(Clone, Debug, Event)]
pub struct ItemUnequipped {
    pub item_object_id: ObjectId,
    pub skip_db_update: bool,
}

impl ItemUnequipped {
    pub fn new(item_object_id: ObjectId, skip_db_update: bool) -> Self {
        Self {
            item_object_id,
            skip_db_update,
        }
    }
}
