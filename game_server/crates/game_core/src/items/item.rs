use super::{
    BodyPart, Id, ItemLocation, ItemLocationVariant, SortingKind,
    augument_id::{AugumentId, EnchantOptions},
    item_info::ItemInfo,
    model::Model,
};
use crate::{
    custom_hierarchy::DespawnChildren,
    items::{self, DollSlot},
    object_id::ObjectId,
    stats::{EncountersVisibility, ItemElementsInfo},
};
use avian3d::prelude::{Collider, CollisionLayers};
use bevy::prelude::*;
use bevy_defer::{AccessError, AsyncAccess, AsyncWorld};
use derive_more::{From, Into};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    packets::ServerPacketBuffer,
};
use physics::GameLayer;
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use serde::Deserialize;

#[derive(Bundle, Clone, Copy, Debug, Default, From, Into, Reflect)]
pub struct UniqueItem(ObjectId, Item);
impl UniqueItem {
    pub fn new(object_id: ObjectId, item: Item) -> Self {
        Self(object_id, item)
    }

    pub fn from_model(model: Model, item_info: &ItemInfo) -> Self {
        UniqueItem::new(model.object_id(), Item::from_model(model, item_info))
    }

    pub fn object_id(&self) -> ObjectId {
        self.0
    }

    pub fn item(&self) -> &Item {
        &self.1
    }

    pub fn item_mut(&mut self) -> &mut Item {
        &mut self.1
    }

    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut buffer = ServerPacketBuffer::default();

        buffer.u32(self.0.into());
        buffer.u32(self.1.id.into());
        buffer.u32(self.1.location.location_data());
        buffer.u64(self.1.count);
        buffer.u16(self.1.sorting_kind.into());
        buffer.u16(self.1.custom_type1);
        buffer.u16_from_bool(self.1.equipped());
        buffer.u32(self.1.bodypart.map_or(0, |bp| bp.into()));
        buffer.u16(self.1.enchant_level);
        buffer.u16(self.1.custom_type2);
        buffer.u32(self.1.augumentation_id.into());
        buffer.i32(self.1.mana.unwrap_or(0));
        buffer.i32(self.1.time.unwrap_or(0));
        buffer.extend(self.1.elements.to_le_bytes());
        buffer.extend(self.1.enchant_options.to_le_bytes());
        buffer.into()
    }
}

impl std::fmt::Display for UniqueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>: {}", self.0, self.1.id())
    }
}

impl PartialEq for UniqueItem {
    fn eq(&self, other: &UniqueItem) -> bool {
        self.0 == other.0
    }
}

#[derive(Bundle)]
pub struct ItemInWorld {
    pub transform: Transform,
    pub collider: Collider,
    pub layers: CollisionLayers,
}

impl ItemInWorld {
    const COLLIDER_SIZE: f32 = 1.0;

    pub fn new(location: Vec3) -> Self {
        Self {
            transform: Transform::from_translation(location),
            collider: Collider::cuboid(
                Self::COLLIDER_SIZE,
                Self::COLLIDER_SIZE,
                Self::COLLIDER_SIZE,
            ),
            layers: GameLayer::item(),
        }
    }

    pub fn move_from_world(mut entity: EntityCommands) {
        entity.remove::<(
            Transform,
            GlobalTransform,
            Collider,
            CollisionLayers,
            DespawnChildren,
        )>();
    }
}

#[derive(Clone, Component, Copy, Debug, Deserialize, Reflect)]
#[require(Name::new("Item"), EncountersVisibility::default())]
pub struct Item {
    id: Id,
    display_id: Id,
    location: super::ItemLocation,
    bodypart: Option<BodyPart>,
    sorting_kind: SortingKind,
    owner: Option<ObjectId>,
    dropped_entity: Option<Entity>,
    count: u64,
    prev_count: u64,
    time: Option<i32>,
    enchant_level: u16,
    mana: Option<i32>,
    drop_time: Option<u64>,
    augumentation_id: AugumentId,
    enchant_options: EnchantOptions,
    elements: ItemElementsInfo,
    // dont know what this is yet
    custom_type1: u16,
    custom_type2: u16,
}

impl Item {
    pub async fn update_count_in_database(&self, object_id: ObjectId) -> Result<(), AccessError> {
        let Ok(items_repository) = AsyncWorld
            .resource::<RepositoryManager>()
            .get(|registry| registry.typed::<ObjectId, items::model::Entity>())?
        else {
            return Ok(());
        };

        let item_model = Model::from(UniqueItem::new(object_id, *self));
        let count = self.count();
        if count == 0 {
            // Delete item from database when count is 0
            items_repository.delete(&item_model).await?;
        } else {
            // Update count in database
            let mut item_model = item_model.into_active_model();
            item_model.count = Set(count as i64);
            items_repository.update(&item_model).await?;
        }
        Ok(())
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: Id::default(),
            display_id: Id::default(),
            location: super::ItemLocation::Unknown,
            bodypart: None,
            sorting_kind: SortingKind::Item,
            owner: None,
            dropped_entity: None,
            count: 1,
            prev_count: 1,
            time: None,
            enchant_level: 0,
            mana: None,
            drop_time: None,
            augumentation_id: AugumentId::default(),
            custom_type1: 0,
            custom_type2: 0,
            enchant_options: EnchantOptions::default(),
            elements: ItemElementsInfo::default(),
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Item {
    pub fn new(id: Id, location: ItemLocation, item_info: &ItemInfo) -> Self {
        Self {
            id,
            display_id: item_info.display_id().unwrap_or(id),
            location,
            bodypart: item_info.bodypart(),
            sorting_kind: item_info.sorting_kind(),
            owner: None,
            dropped_entity: None,
            count: 1,
            prev_count: 1,
            time: None,
            enchant_level: 0,
            mana: None,
            drop_time: None,
            augumentation_id: AugumentId::default(),
            custom_type1: 0,
            custom_type2: 0,
            enchant_options: EnchantOptions::default(),
            elements: ItemElementsInfo::default(),
        }
    }

    pub fn new_with_count(
        id: Id,
        count: u64,
        location: ItemLocation,
        item_info: &ItemInfo,
    ) -> Self {
        Self {
            id,
            display_id: item_info.display_id().unwrap_or(id),
            location,
            bodypart: item_info.bodypart(),
            sorting_kind: item_info.sorting_kind(),
            owner: None,
            dropped_entity: None,
            count,
            prev_count: count,
            time: None,
            enchant_level: 0,
            mana: None,
            drop_time: None,
            augumentation_id: AugumentId::default(),
            custom_type1: 0,
            custom_type2: 0,
            enchant_options: EnchantOptions::default(),
            elements: ItemElementsInfo::default(),
        }
    }

    pub fn from_model(model: Model, item_info: &ItemInfo) -> Self {
        Self {
            id: model.item_id(),
            display_id: item_info.display_id().unwrap_or(model.item_id()),
            location: model.build_location(),
            bodypart: item_info.bodypart(),
            sorting_kind: item_info.sorting_kind(),
            owner: model.owner_id(),
            dropped_entity: None,
            count: model.count(),
            prev_count: model.count(),
            time: model.time(),
            enchant_level: model.enchant_level(),
            mana: model.mana(),
            drop_time: None,
            augumentation_id: AugumentId::default(),
            custom_type1: 0,
            custom_type2: 0,
            enchant_options: EnchantOptions::default(),
            elements: model.elements(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn set_owner(&mut self, object_id: Option<ObjectId>) {
        self.owner = object_id;
    }

    pub fn owner(&self) -> Option<ObjectId> {
        self.owner
    }

    pub fn location(&self) -> super::ItemLocation {
        self.location
    }

    pub fn set_location(&mut self, location: super::ItemLocation) {
        self.location = location;
    }

    pub fn set_dropped_entity(&mut self, entity: Option<Entity>) {
        self.dropped_entity = entity;
    }

    pub fn dropped_entity(&self) -> Option<Entity> {
        self.dropped_entity
    }

    pub fn equipped(&self) -> bool {
        ItemLocationVariant::PaperDoll == self.location.into()
    }

    pub fn bodypart(&self) -> Option<BodyPart> {
        self.bodypart
    }

    pub fn augumentation_id(&self) -> AugumentId {
        self.augumentation_id
    }

    pub fn equip(&mut self, doll_slot: DollSlot) {
        self.location = super::ItemLocation::PaperDoll(doll_slot);
    }

    pub fn unequip(&mut self) {
        self.location = super::ItemLocation::Inventory;
    }

    pub fn set_count(&mut self, count: u64) {
        self.count = count;
    }

    pub fn dec_count(&mut self) {
        self.count -= 1;
    }

    pub fn set_prev_count(&mut self, count: u64) {
        self.prev_count = count;
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn prev_count(&self) -> u64 {
        self.prev_count
    }

    pub fn set_enchant_level(&mut self, level: u16) {
        self.enchant_level = level;
    }

    pub fn enchant_level(&self) -> u16 {
        self.enchant_level
    }

    pub fn time(&self) -> Option<i32> {
        self.time
    }

    pub fn mana(&self) -> Option<i32> {
        self.mana
    }

    pub fn sorting_kind(&self) -> SortingKind {
        self.sorting_kind
    }

    pub fn set_drop_time(&mut self, time: u64) {
        self.drop_time = Some(time);
    }

    pub fn drop_time(&self) -> Option<u64> {
        self.drop_time
    }

    pub fn is_time_limited_item(&self) -> bool {
        self.time.is_some()
    }

    pub fn elements(&self) -> &ItemElementsInfo {
        &self.elements
    }
}

pub trait UsableItem {
    fn usable(&self) -> bool;
}
