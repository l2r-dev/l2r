use crate::{action::model::ActionId, object_id::ObjectId, skills};
use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{self, ValueType, ValueTypeErr},
};
use strum::{Display, EnumDiscriminants, EnumIter};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, EnumDiscriminants, PartialEq, Reflect)]
#[strum_discriminants(name(ShortcutKindVariant))]
#[strum_discriminants(derive(
    Display,
    EnumIter,
    TryFromPrimitive,
    IntoPrimitive,
    Reflect,
    Default
))]
pub enum ShortcutKind {
    #[default]
    #[strum_discriminants(default)]
    None,
    Item(ObjectId),
    Skill(skills::Id, skills::Level),
    Action(ActionId),
    Macro(u32),
    Recipe(u32),
    Bookmark(u32),
}

impl ShortcutKind {
    pub fn new(
        variant: ShortcutKindVariant,
        value: u32,
        level: skills::Level,
    ) -> Result<Self, String> {
        match variant {
            ShortcutKindVariant::None => Ok(ShortcutKind::None),
            ShortcutKindVariant::Item => Ok(ShortcutKind::Item(ObjectId::from(value))),
            ShortcutKindVariant::Skill => Ok(ShortcutKind::Skill(skills::Id::from(value), level)),
            ShortcutKindVariant::Action => Ok(ShortcutKind::Action(ActionId::try_from(value)?)),
            ShortcutKindVariant::Macro => Ok(ShortcutKind::Macro(value)),
            ShortcutKindVariant::Recipe => Ok(ShortcutKind::Recipe(value)),
            ShortcutKindVariant::Bookmark => Ok(ShortcutKind::Bookmark(value)),
        }
    }

    pub fn variant(&self) -> ShortcutKindVariant {
        ShortcutKindVariant::from(*self)
    }
}

impl From<ShortcutKindVariant> for Value {
    fn from(kind: ShortcutKindVariant) -> Self {
        Value::SmallInt(Some(kind as i16))
    }
}

impl TryGetable for ShortcutKindVariant {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i16 = res.try_get_by(idx)?;
        <ShortcutKindVariant as std::convert::TryFrom<u32>>::try_from(value as u32).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to convert {value} to ShortcutKindVariant enum"
            )))
        })
    }
}

impl ValueType for ShortcutKindVariant {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::SmallInt(Some(value)) = v {
            <ShortcutKindVariant as std::convert::TryFrom<u32>>::try_from(value as u32)
                .map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(ShortcutKindVariant).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::SmallInteger
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::SmallInt
    }
}
