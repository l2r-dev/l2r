use crate::{character, object_id::ObjectId, skills, stats::SubClassVariant};
use bevy::prelude::*;
use l2r_core::db::{DbRepository, PrimaryKeyColumns, RepositoryModel, UpdatableModel};
use sea_orm::{Condition, entity::prelude::*, sea_query::SimpleExpr};

pub type CharacterSkillsRepository = DbRepository<SkillPK, Entity>;

#[derive(Clone, Component, Debug, Default, DeriveEntityModel, PartialEq, Reflect)]
#[sea_orm(table_name = "character_skills")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub char_id: ObjectId,
    #[sea_orm(primary_key)]
    pub skill_id: skills::Id,
    #[sea_orm(primary_key)]
    pub sub_class: SubClassVariant,
    pub skill_level: i32,
}

impl Model {
    pub fn new(char_id: ObjectId, skill: skills::Skill, sub_class: SubClassVariant) -> Self {
        Self {
            char_id,
            skill_id: skill.id(),
            skill_level: skill.level().into(),
            sub_class,
        }
    }
}

impl UpdatableModel for Model {
    type Column = Column;

    fn update_columns() -> &'static [Self::Column] {
        &[Column::SkillLevel]
    }
}

impl PrimaryKeyColumns for Model {
    type Column = Column;

    fn pk_columns() -> &'static [Self::Column] {
        &[Column::CharId, Column::SkillId, Column::SubClass]
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

impl Related<character::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Character.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillPK {
    pub char_id: ObjectId,
    pub skill_id: skills::Id,
    pub sub_class: SubClassVariant,
}

impl From<&Model> for SkillPK {
    fn from(model: &Model) -> Self {
        SkillPK {
            char_id: model.char_id,
            skill_id: model.skill_id,
            sub_class: model.sub_class,
        }
    }
}

impl From<SkillPK> for Condition {
    fn from(pk: SkillPK) -> Self {
        Condition::all()
            .add(Column::CharId.eq(pk.char_id))
            .add(Column::SkillId.eq(pk.skill_id))
            .add(Column::SubClass.eq(pk.sub_class))
    }
}

impl From<SkillPK> for SimpleExpr {
    fn from(value: SkillPK) -> Self {
        Column::CharId
            .eq(value.char_id)
            .and(Column::SkillId.eq(value.skill_id))
            .and(Column::SubClass.eq(value.sub_class))
    }
}

impl From<SkillPK> for (ObjectId, skills::Id, SubClassVariant) {
    fn from(pk: SkillPK) -> Self {
        (pk.char_id, pk.skill_id, pk.sub_class)
    }
}
