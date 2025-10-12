use crate::{character, object_id::ObjectId, stats::SubClassVariant};
use bevy::prelude::*;
use l2r_core::db::{DbRepository, PrimaryKeyColumns, RepositoryModel, UpdatableModel};
use sea_orm::{Condition, entity::prelude::*, sea_query::SimpleExpr};

pub type CharacterShortcutsRepository = DbRepository<ShortcutPK, Entity>;

#[derive(Clone, Copy, Debug, DeriveEntityModel, PartialEq, Reflect)]
#[sea_orm(table_name = "character_shortcuts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub char_id: ObjectId,
    #[sea_orm(primary_key)]
    pub slot_id: super::SlotId,
    #[sea_orm(primary_key)]
    pub class_variant: SubClassVariant,
    pub kind: super::ShortcutKindVariant,
    pub shortcut_id: i32,
    pub level: Option<i32>,
}

impl PrimaryKeyColumns for Model {
    type Column = Column;

    fn pk_columns() -> &'static [Self::Column] {
        &[Column::CharId, Column::SlotId, Column::ClassVariant]
    }
}

impl UpdatableModel for Model {
    type Column = Column;

    fn update_columns() -> &'static [Self::Column] {
        &[Column::Kind, Column::ShortcutId]
    }
}

impl RepositoryModel for Model {}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "character::model::Entity",
        from = "Column::CharId",
        to = "character::model::Column::Id"
    )]
    Character,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShortcutPK {
    pub char_id: ObjectId,
    pub slot_id: super::SlotId,
    pub class_variant: SubClassVariant,
}

impl From<&Model> for ShortcutPK {
    fn from(model: &Model) -> Self {
        ShortcutPK {
            char_id: model.char_id,
            slot_id: model.slot_id,
            class_variant: model.class_variant,
        }
    }
}

impl From<ShortcutPK> for Condition {
    fn from(pk: ShortcutPK) -> Self {
        Condition::all()
            .add(Column::CharId.eq(pk.char_id))
            .add(Column::SlotId.eq(pk.slot_id))
            .add(Column::ClassVariant.eq(pk.class_variant))
    }
}

impl From<ShortcutPK> for SimpleExpr {
    fn from(value: ShortcutPK) -> Self {
        Column::CharId
            .eq(value.char_id)
            .and(Column::SlotId.eq(value.slot_id))
            .and(Column::ClassVariant.eq(value.class_variant))
    }
}

impl From<ShortcutPK> for (ObjectId, super::SlotId, SubClassVariant) {
    fn from(pk: ShortcutPK) -> Self {
        (pk.char_id, pk.slot_id, pk.class_variant)
    }
}
