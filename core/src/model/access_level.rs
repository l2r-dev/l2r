use bevy::reflect::Reflect;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::{
    ColumnType, TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Reflect,
    IntoPrimitive,
    TryFromPrimitive,
    EnumIter,
    Display,
    Serialize,
    Deserialize,
)]
#[repr(i16)]
pub enum AccessLevel {
    //FIXME: make everybody admins for testing #[default]
    Player,
    SupportGM,
    EventGM,
    HeadGM,
    #[default]
    Admin,
}

impl AccessLevel {
    pub fn gm(&self) -> bool {
        matches!(
            self,
            AccessLevel::SupportGM
                | AccessLevel::EventGM
                | AccessLevel::HeadGM
                | AccessLevel::Admin
        )
    }

    pub fn admin(&self) -> bool {
        matches!(self, AccessLevel::Admin)
    }
}

impl From<AccessLevel> for Value {
    fn from(level: AccessLevel) -> Self {
        Value::SmallInt(Some(i16::from(level)))
    }
}

impl TryGetable for AccessLevel {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i16 = res.try_get_by(idx)?;
        <AccessLevel as TryFrom<i16>>::try_from(value)
            .map_err(|_| TryGetError::Null("Cant't convert to AccessLevel".to_string()))
    }
}

impl ValueType for AccessLevel {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::SmallInt(Some(val)) => {
                <AccessLevel as TryFrom<i16>>::try_from(val).map_err(|_| ValueTypeErr)
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(AccessLevel).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::SmallInteger
    }

    fn array_type() -> ArrayType {
        ArrayType::SmallInt
    }
}
