use super::{Id, Item, ItemInfo};
use crate::{
    items::{ITEMS_OPERATION_STACK, UniqueItem},
    object_id::{ObjectId, ObjectIdManager},
};
use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use l2r_core::{assets::ASSET_DIR, chronicles::CHRONICLE, model::generic_number::GenericNumber};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use smallvec::SmallVec;
use std::{fs::File, io::BufReader};

#[derive(Asset, Clone, Debug, Default, Deref, DerefMut, Deserialize, Reflect, Serialize)]
pub struct ItemsInfo(HashMap<super::Id, ItemInfo>);
impl ItemsInfo {
    pub fn merge(&mut self, other: &Self) {
        for (id, item_info) in other.iter() {
            self.insert(*id, item_info.clone());
        }
    }

    pub fn test_data() -> Self {
        let mut asset_dir = l2r_core::utils::get_base_path();
        asset_dir.push(ASSET_DIR);
        let path = asset_dir.join("items").join(CHRONICLE).join("test.json");
        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file {path:?}"));
        let reader = BufReader::new(file);
        let items_list: ItemsInfo = from_reader(reader).expect("Failed to parse items from json");
        items_list
    }
}

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
#[reflect(Resource)]
pub struct ItemsDataTable(HashMap<Id, Handle<ItemsInfo>>);
impl ItemsDataTable {
    pub fn get_item_info<'a>(
        &self,
        id: Id,
        item_data: &'a Assets<ItemsInfo>,
    ) -> Result<&'a ItemInfo> {
        let handle = self
            .get(&id)
            .ok_or_else(|| BevyError::from(format!("No handle found for item ID: {}", id)))?;

        let data = item_data.get(handle.id()).ok_or_else(|| {
            BevyError::from(format!("No item data found for handle: {:?}", handle.id()))
        })?;

        data.get(&id)
            .ok_or_else(|| BevyError::from(format!("No item info found for ID: {}", id)))
    }

    pub fn test_data(asset_server: &AssetServer) -> Self {
        let mut asset_dir = l2r_core::utils::get_base_path();
        asset_dir.push(ASSET_DIR);
        let path = asset_dir.join("tests").join("items.json");
        let handle: Handle<ItemsInfo> = asset_server.load(path);
        let test_id = Id::from(0);
        let range = test_id.range();
        let mut items_data_table = ItemsDataTable::default();
        for id in range {
            items_data_table.insert(id, handle.clone());
        }
        items_data_table
    }
}

pub trait ItemsDataAccess {
    fn item(&self, entity: Entity) -> Result<Ref<'_, Item>>;
    fn item_by_object_id(&self, object_id: ObjectId) -> Result<Ref<'_, Item>>;
    fn entity(&self, object_id: ObjectId) -> Result<Entity>;
    fn item_info(&self, id: Id) -> Result<&ItemInfo>;
    fn info_by_object_id(&self, object_id: ObjectId) -> Result<&ItemInfo>;
    fn unique_items_from_object_ids(
        &self,
        object_ids: &[ObjectId],
    ) -> SmallVec<[UniqueItem; ITEMS_OPERATION_STACK]> {
        let mut unique_items = SmallVec::<[UniqueItem; ITEMS_OPERATION_STACK]>::new();
        for &object_id in object_ids {
            if let Ok(item) = self.item_by_object_id(object_id) {
                let unique_item = UniqueItem::new(object_id, *item);
                unique_items.push(unique_item);
            }
        }
        unique_items
    }
}

#[derive(SystemParam)]
pub struct ItemsDataQuery<'w, 's> {
    items_data_table: Res<'w, ItemsDataTable>,
    items_data_assets: Res<'w, Assets<ItemsInfo>>,
    items: Query<'w, 's, Ref<'static, Item>>,
    object_id_manager: Res<'w, ObjectIdManager>,
}

impl<'w, 's> ItemsDataAccess for ItemsDataQuery<'w, 's> {
    fn entity(&self, object_id: ObjectId) -> Result<Entity> {
        self.object_id_manager.entity_result(object_id)
    }

    fn item(&self, entity: Entity) -> Result<Ref<'_, Item>> {
        Ok(self.items.get(entity)?)
    }

    fn item_by_object_id(&self, object_id: ObjectId) -> Result<Ref<'_, Item>> {
        let entity = self.entity(object_id)?;
        Ok(self.items.get(entity)?)
    }

    fn item_info(&self, id: Id) -> Result<&ItemInfo> {
        self.items_data_table
            .get_item_info(id, &self.items_data_assets)
    }

    fn info_by_object_id(&self, object_id: ObjectId) -> Result<&ItemInfo> {
        let entity = self.entity(object_id)?;
        let item = self.items.get(entity)?;
        self.item_info(item.id())
    }
}

#[derive(SystemParam)]
pub struct ItemsDataQueryMut<'w, 's> {
    items_data_table: Res<'w, ItemsDataTable>,
    items_data_assets: Res<'w, Assets<ItemsInfo>>,
    items: Query<'w, 's, Mut<'static, Item>>,
    pub object_ids: Query<'w, 's, Ref<'static, ObjectId>>,
    pub object_id_manager: ResMut<'w, ObjectIdManager>,
}

impl<'w, 's> ItemsDataQueryMut<'w, 's> {
    pub fn item_by_object_id_mut(&mut self, object_id: ObjectId) -> Result<Mut<'_, Item>> {
        let entity = self.object_id_manager.entity_result(object_id)?;
        Ok(self.items.get_mut(entity)?)
    }

    pub fn item_mut(&mut self, entity: Entity) -> Result<Mut<'_, Item>> {
        Ok(self.items.get_mut(entity)?)
    }
}

impl<'w, 's> ItemsDataAccess for ItemsDataQueryMut<'w, 's> {
    fn entity(&self, object_id: ObjectId) -> Result<Entity> {
        self.object_id_manager.entity_result(object_id)
    }

    fn item(&self, entity: Entity) -> Result<Ref<'_, Item>> {
        Ok(self.items.get(entity)?)
    }

    fn item_by_object_id(&self, object_id: ObjectId) -> Result<Ref<'_, Item>> {
        Ok(self.items.get(self.entity(object_id)?)?)
    }

    fn item_info(&self, id: Id) -> Result<&ItemInfo> {
        self.items_data_table
            .get_item_info(id, &self.items_data_assets)
    }

    fn info_by_object_id(&self, object_id: ObjectId) -> Result<&ItemInfo> {
        let entity = self.entity(object_id)?;
        let item = self.items.get(entity)?;
        self.item_info(item.id())
    }
}

impl<'w, 's> IntoIterator for &'w ItemsDataQuery<'w, 's> {
    type Item = (Id, &'w ItemInfo);
    type IntoIter = ItemsDataIterator<'w>;

    fn into_iter(self) -> Self::IntoIter {
        ItemsDataIterator::new(&self.items_data_table, &self.items_data_assets)
    }
}

pub struct ItemsDataIterator<'a> {
    table_iter: bevy::platform::collections::hash_map::Iter<'a, Id, Handle<ItemsInfo>>,
    items_data_assets: &'a Assets<ItemsInfo>,
}

impl<'a> ItemsDataIterator<'a> {
    fn new(table: &'a ItemsDataTable, items_data_assets: &'a Assets<ItemsInfo>) -> Self {
        Self {
            table_iter: table.iter(),
            items_data_assets,
        }
    }
}

impl<'a> Iterator for ItemsDataIterator<'a> {
    type Item = (Id, &'a ItemInfo);

    fn next(&mut self) -> Option<Self::Item> {
        for (item_id, item_handle) in self.table_iter.by_ref() {
            if let Some(items_info) = self.items_data_assets.get(item_handle.id()) {
                if let Some(item_info) = items_info.get(item_id) {
                    return Some((*item_id, item_info));
                }
            }
        }
        None
    }
}
