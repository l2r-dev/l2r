use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_defer::{AccessError, AppReactorExtension, AsyncAccess, AsyncExtension, AsyncWorld};
use game_core::{
    custom_hierarchy::DespawnChildOf,
    items::{
        self, AddInInventory, Inventory, Item, ItemInWorld, ItemLocation, ItemMetric,
        ItemsComponentsPlugin, ItemsDataAccess, ItemsDataQuery, ItemsInfo, SilentSpawn,
        SpawnExisting, SpawnNew, UniqueItem, model,
    },
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId},
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    metrics::MetricsAppExt,
};
use map::WorldMap;
use state::{GameMechanicsSystems, LoadingSystems};
use use_shot::UseShotPlugin;

mod admin_shop;
mod assets;
mod inventory;
mod item;
mod request_destroy_item;
mod request_drop_item;
mod request_item_list;
mod use_item;
mod use_shot;

pub use inventory::*;
pub use item::*;

pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ItemsComponentsPlugin)
            .add_plugins(request_item_list::RequestItemListPlugin)
            .add_plugins(request_drop_item::RequestDropItemPlugin)
            .add_plugins(request_destroy_item::RequestDestroyItemPlugin)
            .add_plugins(use_item::UseItemPlugin)
            .add_plugins(InventoryPlugin)
            .add_plugins(UseShotPlugin)
            .add_plugins(admin_shop::AdminShopPlugin)
            .add_plugins(JsonAssetPlugin::<ItemsInfo>::new(&["json"]));

        app.register_counter(ItemMetric::ItemsDropped, "Total items dropped");

        app.spawn_task(async { spawn_new_items().await })
            .react_to_event::<SpawnNew>();

        app.add_systems(
            Update,
            (
                assets::load_items_data_assets.in_set(LoadingSystems::AssetInit),
                assets::update_items_data_assets.in_set(LoadingSystems::AssetInit),
            ),
        );
        app.add_systems(
            Update,
            assets::update_items_data_assets.in_set(GameMechanicsSystems::Items),
        )
        .add_systems(Update, count_changed.in_set(GameMechanicsSystems::Items))
        .add_systems(
            Update,
            handle_newly_spawned_items.in_set(GameMechanicsSystems::Items),
        );

        app.add_observer(spawn_existing_item_handle);
    }
}

async fn spawn_new_items() -> Result<(), AccessError> {
    while let Ok(spawn_event) = AsyncWorld.get_next_event::<SpawnNew>().await {
        let Ok(items_repository) = AsyncWorld
            .resource::<RepositoryManager>()
            .get(|manager| manager.typed::<ObjectId, items::model::Entity>())?
        else {
            return Ok(());
        };

        let mut created_items = Vec::with_capacity(spawn_event.item_ids.len());

        for &item_id in spawn_event.item_ids.iter() {
            let object_id = AsyncWorld
                .resource::<ObjectIdManager>()
                .get_mut(|object_id_manager| object_id_manager.next_id())?;

            let owner_id = spawn_event.owner.and_then(|owner| {
                AsyncWorld
                    .entity(owner)
                    .component::<ObjectId>()
                    .get(|id| *id)
                    .ok()
            });

            let new_item = model::Model::new(
                object_id,
                item_id,
                spawn_event.count,
                spawn_event.item_location,
                owner_id,
            );

            let result = items_repository.create(&new_item).await;
            match result {
                Ok(created_item) => created_items.push(created_item),
                Err(e) => {
                    error!("{}: Failed to add Item to DB: {:?}", stringify!(Self), e);
                }
            }
        }

        if !created_items.is_empty() {
            let silent = spawn_event.silent;
            AsyncWorld.apply_command(move |world: &mut World| {
                world.trigger(SpawnExisting {
                    item_models: created_items,
                    dropped_entity: spawn_event.dropped_entity,
                    silent,
                });
            });
        }
    }
    Ok(())
}

pub fn spawn_existing_item_handle(
    spawn: Trigger<SpawnExisting>,
    mut commands: Commands,
    items_data: ItemsDataQuery,
) {
    let event = spawn.event();

    for item in event.item_models.iter() {
        let Ok(item_info) = items_data.item_info(item.item_id()) else {
            warn!(
                "{}: Item info not found for item ID {}",
                stringify!(SpawnExisting),
                item.item_id()
            );
            return;
        };

        let mut unique_item = UniqueItem::from_model(*item, item_info);
        unique_item
            .item_mut()
            .set_dropped_entity(event.dropped_entity);

        let mut entity_commands = unique_item.spawn(&mut commands, item_info);
        if event.silent {
            entity_commands.insert(SilentSpawn);
        }
    }
}

fn handle_newly_spawned_items(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    object_id_manager: Res<ObjectIdManager>,
    inventory_entities: Query<Entity, With<Inventory>>,
    newly_spawned_items: Query<(Entity, Ref<ObjectId>, Ref<Item>, Has<SilentSpawn>), Added<Item>>,
) -> Result<()> {
    for (item_entity, item_oid, item, silent) in &newly_spawned_items {
        match item.location() {
            ItemLocation::World(translation) => {
                if let Some(region_entity) = world_map.get_by_loc(translation) {
                    commands
                        .entity(item_entity)
                        .insert(DespawnChildOf(region_entity));
                }
                commands
                    .entity(item_entity)
                    .insert(ItemInWorld::new(translation));
            }
            ItemLocation::Inventory | ItemLocation::PaperDoll(_) => {
                let Some(inventory_object_id) = item.owner() else {
                    warn!("Spawned item {:?} does not have an owner ID", item_oid);
                    continue;
                };

                let inventory_entity = match inventory_entities
                    .by_object_id(inventory_object_id, object_id_manager.as_ref())
                {
                    Ok(entity) => entity,
                    Err(err) => {
                        warn!(
                            "Failed to get inventory entity for object_id: {:?}, error: {:?}",
                            inventory_object_id, err
                        );
                        continue;
                    }
                };

                if silent {
                    commands.entity(item_entity).remove::<SilentSpawn>();
                }

                commands.trigger_targets(
                    AddInInventory {
                        item: item_entity,
                        silent,
                    },
                    inventory_entity,
                );
            }
            _ => {}
        }
    }
    Ok(())
}
