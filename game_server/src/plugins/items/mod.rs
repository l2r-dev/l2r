use avian3d::prelude::{Collider, Sensor};
use bevy::{log, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_defer::{AccessError, AppReactorExtension, AsyncAccess, AsyncExtension, AsyncWorld};
use game_core::{
    custom_hierarchy::{DespawnChildOf, DespawnChildren},
    items::{
        self, AddInInventory, Inventory, Item, ItemLocation, ItemMetric, ItemsComponentsPlugin,
        ItemsDataTable, ItemsInfo, RegionalItems, SilentSpawn, SpawnExisting, SpawnNew, UniqueItem,
        model,
    },
    object_id::{ObjectId, ObjectIdManager, QueryByObjectId},
    stats::EncountersVisibility,
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    metrics::MetricsAppExt,
};
use map::{WorldMap, id::RegionId};
use smallvec::smallvec;
use state::GameMechanicsSystems;
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

        app.add_systems(Startup, assets::load_items_data_assets)
            .add_systems(Update, assets::update_items_data_assets)
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
                    log::error!("{}: Failed to add Item to DB: {:?}", stringify!(Self), e);
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
    items_data_table: Res<ItemsDataTable>,
    items_data: Res<Assets<ItemsInfo>>,
) {
    let event = spawn.event();

    for item in event.item_models.iter() {
        let Ok(item_info) = items_data_table.get_item_info(item.item_id(), &items_data) else {
            log::warn!(
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

        let mut entity_commands = commands.spawn((
            unique_item,
            EncountersVisibility::default(),
            Sensor,
            Collider::cuboid(1., 1., 1.),
        ));
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
    region_children: Query<&DespawnChildren>,
    regional_items: Query<Entity, With<RegionalItems>>,
) {
    for (item_entity, item_oid, item, is_silent) in &newly_spawned_items {
        if is_silent {
            commands.entity(item_entity).remove::<SilentSpawn>();
        }

        match item.location() {
            ItemLocation::World(translation) => {
                if let Some(region_entity) = world_map.get(&RegionId::from(translation))
                    && let Ok(region_children_list) = region_children.get(*region_entity)
                {
                    let regional_items_entity = region_children_list
                        .iter()
                        .find(|child| regional_items.get(*child).is_ok());

                    if let Some(regional_items_entity) = regional_items_entity {
                        commands
                            .entity(item_entity)
                            .insert(DespawnChildOf(regional_items_entity));
                    }
                }
                commands
                    .entity(item_entity)
                    .insert(Transform::from_translation(translation));
            }
            ItemLocation::Inventory | ItemLocation::PaperDoll(_) => {
                let Some(inventory_object_id) = item.owner() else {
                    log::warn!("Spawned item {:?} does not have an owner ID", item_oid);
                    continue;
                };

                let inventory_entity = match inventory_entities
                    .by_object_id(inventory_object_id, object_id_manager.as_ref())
                {
                    Ok(entity) => entity,
                    Err(err) => {
                        log::warn!(
                            "Failed to get inventory entity for object_id: {:?}, error: {:?}",
                            inventory_object_id,
                            err
                        );
                        continue;
                    }
                };

                let add_inventory_event = if is_silent {
                    AddInInventory::new_silent(smallvec![item_entity])
                } else {
                    AddInInventory::new(smallvec![item_entity])
                };
                commands.trigger_targets(add_inventory_event, inventory_entity);
            }
            _ => {}
        }
    }
}
