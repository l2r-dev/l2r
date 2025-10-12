use bevy::prelude::*;
use game_core::{
    items::{Id, ItemsDataTable, ItemsInfo},
    multisell::admin_shop::AdminShopUpdate,
};
use l2r_core::{
    assets::ASSET_DIR, chronicles::CHRONICLE, model::generic_number::GenericNumber as _,
};
use std::path::PathBuf;

const ITEMS_PATH: &str = "items";

pub(super) fn load_items_data_assets(
    asset_server: Res<AssetServer>,
    mut items_data_table: ResMut<ItemsDataTable>,
) {
    let mut asset_dir = l2r_core::utils::get_base_path();
    asset_dir.push(ASSET_DIR);
    if let Ok(entries) = std::fs::read_dir(asset_dir.join(ITEMS_PATH).join(CHRONICLE)) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file()
                && entry_path.extension().and_then(|s| s.to_str()) == Some("json")
            {
                let filename = entry_path.file_name().unwrap().to_str().unwrap();
                let mut path = PathBuf::from(format!("{ITEMS_PATH}/{CHRONICLE}/"));
                path.push(filename);
                let handle: Handle<ItemsInfo> = asset_server.load(path.clone());
                let id_range = Id::range_from_filename(filename);

                for id in id_range {
                    items_data_table.insert(id, handle.clone());
                }
            }
        }
    }
}

pub(super) fn update_items_data_assets(
    mut events: EventReader<AssetEvent<ItemsInfo>>,
    mut events_admin_shop: EventWriter<AdminShopUpdate>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id: _ } | AssetEvent::Modified { id: _ } => {
                events_admin_shop.write(AdminShopUpdate);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::{chronicles::CHRONICLE, utils::get_base_path};
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_items_from_json() {
        let mut asset_dir = get_base_path();
        asset_dir.push(ASSET_DIR);
        let path = asset_dir.join("tests\\items.json");
        println!("{:?}", path);

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file {:?}", path));
        let reader = BufReader::new(file);

        let npc_list: ItemsInfo = from_reader(reader).expect("Failed to parse items from json");

        println!("{:?}", npc_list);
    }

    #[test]
    fn test_parse_all_items_from_json() {
        let mut asset_dir = get_base_path();
        asset_dir.push(ASSET_DIR);
        let items_dir = asset_dir.join(format!("items\\{}\\", CHRONICLE));

        for entry in std::fs::read_dir(items_dir).expect("Failed to read items directory") {
            let entry = entry.expect("Failed to get entry");
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file =
                    File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
                let reader = BufReader::new(file);

                let items_list: ItemsInfo = from_reader(reader)
                    .unwrap_or_else(|_| panic!("Failed to parse items from JSON: {:?}", path));

                assert!(!items_list.is_empty());
            }
        }
    }
}
