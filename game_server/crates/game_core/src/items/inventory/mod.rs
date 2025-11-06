use crate::{
    character::Character,
    items::{Id, Item},
    object_id::{ObjectId, ObjectIdIndexSet, ObjectIdManager},
};
use bevy::prelude::*;
use bevy_ecs::system::SystemParam;
use derive_more::{From, Into};
use num_enum::IntoPrimitive;

mod events;
mod paperdoll;

pub use events::*;
pub use paperdoll::*;

pub struct InventoryComponentsPlugin;
impl Plugin for InventoryComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inventory>()
            .register_type::<PaperDoll>()
            .register_type::<DollSlot>();

        app.add_event::<AddInInventory>()
            .add_event::<AddNonStackable>()
            .add_event::<AddStackable>()
            .add_event::<InventoryLoad>()
            .add_event::<DropIfPossible>()
            .add_event::<DropItemEvent>()
            .add_event::<DestroyItemRequest>();
    }
}

#[derive(Clone, Copy, Event, From, Into)]
pub struct InventoryLoad(pub Entity);

#[derive(Deref, SystemParam)]
pub struct CharacterInventories<'w, 's>(Query<'w, 's, Ref<'static, Inventory>, With<Character>>);

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Inventory(ObjectIdIndexSet);
impl Inventory {
    pub fn get_item(&self, item: ObjectId) -> Result<ObjectId> {
        if self.0.contains(&item) {
            Ok(item)
        } else {
            Err(BevyError::from(format!(
                "Item {} not found in inventory",
                item
            )))
        }
    }

    pub fn get_by_item_id(
        &self,
        item_id: Id,
        items: &Query<Ref<Item>>,
        object_id_manager: &ObjectIdManager,
    ) -> Vec<(Entity, ObjectId, Id)> {
        self.iter()
            .filter_map(|object_id| {
                let entity = object_id_manager.entity(*object_id)?;
                let item = items.get(entity).ok()?;
                if item.id() == item_id {
                    Some((entity, *object_id, item.id()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn single_by_item_id(
        &self,
        item_id: Id,
        items: &Query<Ref<Item>>,
        object_id_manager: &ObjectIdManager,
    ) -> Option<(Entity, ObjectId, Id)> {
        self.iter().find_map(|object_id| {
            let entity = object_id_manager.entity(*object_id)?;
            let item = items.get(entity).ok()?;
            if item.id() == item_id {
                Some((entity, *object_id, item.id()))
            } else {
                None
            }
        })
    }

    pub fn remove_item(&mut self, item: ObjectId) -> Result<ObjectId> {
        let object_id = self.get_item(item)?;
        self.shift_remove(&object_id);
        Ok(object_id)
    }

    pub fn add_multiple(&mut self, items: &[ObjectId]) {
        for item in items {
            self.insert(*item);
        }
    }
}

#[derive(Clone, Copy, Debug, IntoPrimitive, Reflect)]
#[repr(u16)]
pub enum UpdateType {
    Add = 1,
    Modify = 2,
    Remove = 3,
}
