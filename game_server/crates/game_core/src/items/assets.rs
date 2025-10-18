use super::{Id, ItemInfo, UniqueItem};
use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use l2r_core::{assets::ASSET_DIR, chronicles::CHRONICLE, model::generic_number::GenericNumber};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
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

#[derive(SystemParam)]
pub struct ItemsDataQuery<'w> {
    pub items_data_table: Res<'w, ItemsDataTable>,
    pub items_data_assets: Res<'w, Assets<ItemsInfo>>,
}

impl<'w> ItemsDataQuery<'w> {
    pub fn get_item_info(&self, id: Id) -> Result<&ItemInfo> {
        self.items_data_table
            .get_item_info(id, &self.items_data_assets)
    }

    pub fn item_info_from_uniq(&self, unique_item: &Option<UniqueItem>) -> Option<&ItemInfo> {
        let Some(unique_item) = unique_item else {
            return None;
        };

        self.items_data_table
            .get_item_info(unique_item.item().id(), &self.items_data_assets)
            .ok()
    }
}

pub trait GetItemInfoFromUniqItem {
    fn item_info_from_uniq(&self, unique_item: &Option<UniqueItem>) -> Option<&ItemInfo>;
}

impl GetItemInfoFromUniqItem for (&Assets<ItemsInfo>, &ItemsDataTable) {
    fn item_info_from_uniq(&self, unique_item: &Option<UniqueItem>) -> Option<&ItemInfo> {
        let Some(unique_item) = unique_item else {
            return None;
        };

        self.1.get_item_info(unique_item.item().id(), self.0).ok()
    }
}
