use super::{ItemLocation, ItemLocationVariant, UniqueItem};
use crate::{character, items::DollSlot, object_id::ObjectId, stats::ItemElementsInfo};
use bevy::{log, prelude::*};
use l2r_core::db::{DbRepository, PrimaryKeyColumns, RepositoryModel, UpdatableModel};
// use chrono::{NaiveDateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, Related};
use spatial::GameVec3;

pub type ItemsRepository = DbRepository<ObjectId, Entity>;

#[derive(Clone, Component, Copy, Debug, Default, DeriveEntityModel, PartialEq, Reflect)]
#[sea_orm(table_name = "items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub object_id: ObjectId,
    pub owner_id: Option<ObjectId>,
    pub item_id: super::Id,
    pub count: i64,
    pub enchant_level: i16,
    pub location: super::ItemLocationVariant,
    pub location_data: i32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z: Option<i32>,
    pub mana: Option<i32>,
    pub time: Option<i32>,
    pub elements_info: Option<ItemElementsInfo>,
}

impl PrimaryKeyColumns for Model {
    type Column = Column;

    fn pk_columns() -> &'static [Self::Column] {
        &[Column::ObjectId]
    }
}

impl UpdatableModel for Model {
    type Column = Column;

    fn update_columns() -> &'static [Self::Column] {
        &[
            Column::OwnerId,
            Column::Count,
            Column::EnchantLevel,
            Column::Location,
            Column::LocationData,
            Column::X,
            Column::Y,
            Column::Z,
            Column::Mana,
            Column::Time,
            Column::ElementsInfo,
        ]
    }
}

impl RepositoryModel for Model {}

impl Model {
    pub fn new(
        object_id: ObjectId,
        item_id: super::Id,
        count: u64,
        location: ItemLocation,
        owner_id: Option<ObjectId>,
    ) -> Self {
        let (x, y, z) = match location {
            ItemLocation::World(translation) => {
                let game_vec = GameVec3::from(translation);
                (Some(game_vec.x), Some(game_vec.y), Some(game_vec.z))
            }
            _ => (None, None, None),
        };

        let location_data = location.location_data() as i32;

        let location = ItemLocationVariant::from(location);

        Self {
            object_id,
            item_id,
            owner_id,
            count: count as i64,
            enchant_level: 0,
            location,
            location_data,
            x,
            y,
            z,
            ..Default::default()
        }
    }

    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }

    pub fn item_id(&self) -> super::Id {
        self.item_id
    }

    pub fn count(&self) -> u64 {
        self.count as u64
    }

    pub fn enchant_level(&self) -> u16 {
        self.enchant_level as u16
    }

    pub fn location(&self) -> super::ItemLocationVariant {
        self.location
    }

    pub fn build_location(&self) -> ItemLocation {
        match self.location {
            super::ItemLocationVariant::World => {
                let game_vec = GameVec3::new(
                    self.x.unwrap_or_default(),
                    self.y.unwrap_or_default(),
                    self.z.unwrap_or_default(),
                );
                ItemLocation::World(Vec3::from(game_vec))
            }
            super::ItemLocationVariant::Inventory => ItemLocation::Inventory,
            super::ItemLocationVariant::PaperDoll => ItemLocation::PaperDoll(
                (self.location_data as u32).try_into().unwrap_or_else(|_| {
                    log::warn!(
                        "Invalid DollSlot conversion for item_oid: {}",
                        self.object_id
                    );
                    DollSlot::Underwear
                }),
            ),
            _ => ItemLocation::Unknown,
        }
    }

    pub fn coordinates(&self) -> Option<Vec3> {
        if self.location == super::ItemLocationVariant::World {
            let game_vec = GameVec3::new(
                self.x.unwrap_or_default(),
                self.y.unwrap_or_default(),
                self.z.unwrap_or_default(),
            );
            Some(Vec3::from(game_vec))
        } else {
            None
        }
    }

    pub fn owner_id(&self) -> Option<ObjectId> {
        self.owner_id
    }

    pub fn set_owner_id(&mut self, owner_id: ObjectId) {
        self.owner_id = Some(owner_id);
    }

    pub fn time(&self) -> Option<i32> {
        self.time
    }

    pub fn mana(&self) -> Option<i32> {
        self.mana
    }

    pub fn elements(&self) -> ItemElementsInfo {
        self.elements_info.unwrap_or_default()
    }

    pub fn equipped(&self) -> bool {
        matches!(self.location, super::ItemLocationVariant::PaperDoll)
    }
}

impl From<UniqueItem> for Model {
    fn from(unique_item: UniqueItem) -> Self {
        let object_id = unique_item.object_id();
        let item = unique_item.item();
        let item_location = item.location();

        let location = match item_location {
            super::ItemLocation::World(translation) => Some(GameVec3::from(translation)),
            _ => None,
        };

        let (x, y, z) = match location {
            Some(location) => (Some(location.x), Some(location.y), Some(location.z)),
            None => (None, None, None),
        };

        let location_data = item_location.location_data() as i32;

        let item_elements = item.elements();
        let mut elements_info = None;
        if item_elements.attack_element.is_some() || item_elements.defence_elements.is_some() {
            elements_info = Some(*item_elements);
        }

        Self {
            object_id,
            item_id: item.id(),
            count: item.count() as i64,
            enchant_level: item.enchant_level() as i16,
            location: item_location.into(),
            location_data,
            x,
            y,
            z,
            mana: item.mana(),
            time: item.time(),
            elements_info,
            ..Default::default()
        }
    }
}

// Example
// let item = items::Entity::find_by_id(item_id).one(&db).await?;
// if let Some(item_model) = item {
//     let character = item_model.find_related(character::Entity).one(&db).await?;
// }
// let items = character::Entity::find_by_id(character_id)
//     .find_with_related(items::Entity)
//     .all(&db)
//     .await?;

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "character::model::Entity",
        from = "Column::OwnerId",
        to = "character::model::Column::Id"
    )]
    Character,
}

impl Related<character::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Character.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub trait ActiveModelSetCoordinates {
    fn set_location(&mut self, location: ItemLocation) -> &mut Self;
}

impl ActiveModelSetCoordinates for ActiveModel {
    fn set_location(&mut self, location: ItemLocation) -> &mut Self {
        let location_variant = ItemLocationVariant::from(location);
        self.location = Set(location_variant);
        self.location_data = Set(location.location_data() as i32);

        match location {
            ItemLocation::World(translation) => {
                let game_vec = GameVec3::from(translation);
                self.x = Set(Some(game_vec.x));
                self.y = Set(Some(game_vec.y));
                self.z = Set(Some(game_vec.z));
            }
            _ => {
                self.x = Set(None);
                self.y = Set(None);
                self.z = Set(None);
            }
        };

        self
    }
}
