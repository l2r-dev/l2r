use crate::{
    items, network::packets::client::RequestCharCreate, object_id::ObjectId, shortcut, stats::*,
    utils::ReflectableDateTime,
};
use bevy::prelude::*;
use l2r_core::{
    db::{DbRepository, PrimaryKeyColumns, RepositoryModel, UpdatableModel},
    model::race::Race,
};
use sea_orm::{ActiveValue::Set, IntoActiveModel, entity::prelude::*};
use spatial::GameVec3;
use std::fmt;

pub type CharacterRepository = DbRepository<ObjectId, Entity>;

#[derive(Clone, Component, Debug, Default, DeriveEntityModel, PartialEq, Reflect)]
#[sea_orm(table_name = "characters")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: ObjectId,
    pub account_id: Uuid,
    pub is_last_active: bool,
    pub name: String,
    pub title: String,
    pub exp: i64,
    pub sp: i32,
    pub created_time: ReflectableDateTime,
    pub race: Race,
    pub sub_class: SubClassVariant,
    pub class_id: ClassId,
    pub appearance: super::Appearance,
    pub vitals: VitalsStats,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl PrimaryKeyColumns for Model {
    type Column = Column;

    fn pk_columns() -> &'static [Self::Column] {
        &[Column::Id]
    }
}

impl UpdatableModel for Model {
    type Column = Column;

    fn update_columns() -> &'static [Self::Column] {
        &[
            Column::Name,
            Column::Title,
            Column::X,
            Column::Y,
            Column::Z,
            Column::Exp,
            Column::Sp,
            Column::Vitals,
            Column::IsLastActive,
        ]
    }
}

impl RepositoryModel for Model {}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Model {
    pub fn new(
        id: ObjectId,
        account_id: Uuid,
        character: RequestCharCreate,
        vitals: VitalsStats,
        position: GameVec3,
    ) -> Self {
        Self {
            id,
            account_id,
            is_last_active: true,
            name: character.name,
            exp: 0,
            sp: 0,
            created_time: ReflectableDateTime::now(),
            race: character.race,
            class_id: character.class_id,
            appearance: character.appearance,
            vitals,
            x: position.x,
            y: position.y,
            z: position.z,
            ..Default::default()
        }
    }

    pub fn update(self, update: ModelUpdate) -> ActiveModel {
        let mut active_model = self.into_active_model();
        active_model.title = Set(update.title);
        active_model.x = Set(update.position.x);
        active_model.y = Set(update.position.y);
        active_model.z = Set(update.position.z);
        active_model.exp = Set(update.exp);
        active_model.sp = Set(update.sp);
        active_model.vitals = Set(update.vitals);
        active_model.is_last_active = Set(update.is_last_active);
        active_model
    }
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        has_many = "items::model::Entity",
        from = "Column::Id",
        to = "items::model::Column::OwnerId"
    )]
    Items,
    #[sea_orm(
        belongs_to = "shortcut::model::Entity",
        from = "Column::Id",
        to = "shortcut::model::Column::CharId"
    )]
    Shortcuts,
    #[sea_orm(
        belongs_to = "super::skills::Entity",
        from = "Column::Id",
        to = "super::skills::Column::CharId"
    )]
    Skills,
}

impl Related<items::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelUpdate {
    pub title: String,
    pub position: GameVec3,
    pub exp: i64,
    pub sp: i32,
    pub vitals: VitalsStats,
    pub is_last_active: bool,
}
