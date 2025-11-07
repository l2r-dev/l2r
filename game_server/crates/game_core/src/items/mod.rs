use crate::object_id::ObjectId;
use bevy::prelude::*;

mod assets;
mod augument_id;
mod bodypart;
mod condition;
mod drop;
mod grade;
mod id;
mod inventory;
mod item;
mod item_info;
pub mod kind;
mod location;
/// Database model related components and types
pub mod model;
mod quality;
mod use_shot;

pub use assets::*;
pub use augument_id::*;
pub use bodypart::BodyPart;
pub use condition::*;
pub use drop::*;
pub use grade::*;
pub use id::Id;
pub use inventory::*;
pub use item::*;
pub use item_info::*;
pub use kind::*;
pub use l2r_core::assets::*;
pub use location::*;
pub use model::ItemsRepository;
pub use quality::*;
pub use use_shot::*;

pub struct ItemsComponentsPlugin;
impl Plugin for ItemsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemsDataTable::default());

        app.register_asset_reflect::<ItemsInfo>()
            .register_type::<Id>()
            .register_type::<model::Model>()
            .register_type::<AugumentId>()
            .register_type::<Item>()
            .register_type::<ItemInfo>()
            .register_type::<ItemLocation>()
            .register_type::<ItemLocationVariant>()
            .register_type::<UniqueItem>()
            .register_type::<ItemsDataTable>()
            .register_type::<RegionalItemsFolder>();

        app.add_event::<UseShot>().add_event::<SpawnNew>();
    }
}

pub const ITEMS_OPERATION_STACK: usize = 3;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(Name::new("Items".to_string()))]
pub struct RegionalItemsFolder;

#[derive(Clone, Debug, Event)]
pub struct SpawnNew {
    pub item_ids: Vec<Id>,
    pub count: u64,
    pub item_location: ItemLocation,
    pub dropped_entity: Option<Entity>,
    pub owner: Option<Entity>,
    pub silent: bool, // If true, no system messages will be sent when items are added to inventory
}

impl SpawnNew {
    pub fn new(
        item_ids: Vec<Id>,
        count: u64,
        item_location: ItemLocation,
        owner: Option<Entity>,
    ) -> Self {
        Self {
            item_ids,
            count,
            item_location,
            dropped_entity: None,
            owner,
            silent: false,
        }
    }

    pub fn new_silent(
        item_ids: Vec<Id>,
        count: u64,
        item_location: ItemLocation,
        owner: Option<Entity>,
    ) -> Self {
        Self {
            item_ids,
            count,
            item_location,
            dropped_entity: None,
            owner,
            silent: true,
        }
    }
}

#[derive(Clone, Debug, Event)]
pub struct SpawnExisting {
    pub item_models: Vec<model::Model>,
    pub dropped_entity: Option<Entity>,
    pub silent: bool, // If true, no system messages will be sent when items are added to inventory
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SilentSpawn;
