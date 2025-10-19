use bevy::prelude::*;
use game_core::{
    items::{Item, ItemLocation, ItemsDataQuery},
    multisell::{Entry, Good, Id, admin_shop::*},
};

pub struct AdminShopPlugin;
impl Plugin for AdminShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AdminShopComponentsPlugin);

        app.add_systems(Update, update_admin_shop_items_list);
    }
}

fn update_admin_shop_items_list(
    mut events: EventReader<AdminShopUpdate>,
    items_data_query: ItemsDataQuery,
    mut admin_shop_items: ResMut<AdminShopMultiSells>,
) {
    if events.read().next().is_none() {
        return;
    } else {
        events.clear();
    }
    // Clear previous items to avoid duplicates, cause assets may be loaded incrementally
    admin_shop_items.clear();
    
    for (item_id, item_handle) in items_data_query.items_data_table.iter() {
        if let Some(items_info) = items_data_query.items_data_assets.get(item_handle.id())
            && let Some(item_info) = items_info.get(item_id)
        {
            let kind = *item_info.kind();
            let grade = item_info.grade();
            let multisell_id = Id::from((kind, grade));
            let item = Item::new_with_count(*item_id, 1, ItemLocation::Store, item_info);
            let entry = Entry {
                stackable: item_info.stackable(),
                rewards: vec![Good::new(item)],
                requirements: Vec::new(),
            };
            admin_shop_items
                .entry(multisell_id)
                .or_default()
                .push(entry);
        }
    }
}
