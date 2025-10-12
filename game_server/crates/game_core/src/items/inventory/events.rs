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
pub struct EquipItems(Entity, pub Vec<ObjectId>);

impl ContainsEntity for EquipItems {
    fn entity(&self) -> Entity {
        self.0
    }
}

impl EquipItems {
    pub fn new(entity: Entity, items: Vec<ObjectId>) -> Self {
        Self(entity, items)
    }

    pub fn object_ids(&self) -> &[ObjectId] {
        self.1.as_slice()
    }
}

#[derive(Debug, Event)]
pub struct ItemsEquipped(Entity, SmallVec<[ObjectId; ITEMS_OPERATION_STACK]>);

impl ContainsEntity for ItemsEquipped {
    fn entity(&self) -> Entity {
        self.0
    }
}

impl ItemsEquipped {
    pub fn new(entity: Entity, items: SmallVec<[ObjectId; ITEMS_OPERATION_STACK]>) -> Self {
        Self(entity, items)
    }

    pub fn object_ids(&self) -> &[ObjectId] {
        self.1.as_slice()
    }
}

#[derive(Clone, Debug, Event)]
pub struct UnequipItems {
    entity: Entity,
    items: Vec<ObjectId>,
    skip_db_update: bool,
}

impl ContainsEntity for UnequipItems {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl UnequipItems {
    pub fn new(entity: Entity, items: Vec<ObjectId>) -> Self {
        Self {
            entity,
            items,
            skip_db_update: false,
        }
    }

    pub fn new_skip_db(entity: Entity, items: Vec<ObjectId>) -> Self {
        Self {
            entity,
            items,
            skip_db_update: true,
        }
    }

    pub fn object_ids(&self) -> &[ObjectId] {
        self.items.as_slice()
    }

    pub fn skip_db_update(&self) -> bool {
        self.skip_db_update
    }
}

#[derive(Clone, Debug, Event)]
pub struct ItemsUnEquipped {
    entity: Entity,
    items: SmallVec<[ObjectId; ITEMS_OPERATION_STACK]>,
    skip_db_update: bool,
}

impl ContainsEntity for ItemsUnEquipped {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl ItemsUnEquipped {
    pub fn new(
        entity: Entity,
        items: SmallVec<[ObjectId; ITEMS_OPERATION_STACK]>,
        skip_db_update: bool,
    ) -> Self {
        Self {
            entity,
            items,
            skip_db_update,
        }
    }

    pub fn object_ids(&self) -> &[ObjectId] {
        self.items.as_slice()
    }

    pub fn skip_db_update(&self) -> bool {
        self.skip_db_update
    }
}
