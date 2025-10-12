use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{self, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    Display,
    EnumIter,
    Component,
    TryFromPrimitive,
    IntoPrimitive,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Default,
    Reflect,
)]
#[repr(u32)]
pub enum Race {
    Human,
    Elf,
    DarkElf,
    Orc,
    Dwarf,
    Kamael,
    Animal,
    Beast,
    Bug,
    CastleGuard,
    Construct,
    Demonic,
    Divine,
    Dragon,
    Elemental,
    #[default]
    Etc,
    Fairy,
    Giant,
    Humanoid,
    Mercenary,
    None,
    Plant,
    SiegeWeapon,
    Undead,
}

impl TryFrom<i32> for Race {
    type Error = num_enum::TryFromPrimitiveError<Self>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Race::try_from_primitive(value as u32)
    }
}

impl Race {
    pub fn default_races() -> Vec<Race> {
        vec![
            Race::Human,
            Race::Elf,
            Race::DarkElf,
            Race::Orc,
            Race::Dwarf,
        ]
    }
}

impl From<Race> for i32 {
    fn from(value: Race) -> Self {
        value as i32
    }
}

impl From<Race> for Value {
    fn from(race: Race) -> Self {
        Value::Int(Some(race as i32))
    }
}

impl TryGetable for Race {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by(idx)?;
        <Race as std::convert::TryFrom<i32>>::try_from(value).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to convert {value} to Race enum"
            )))
        })
    }

    fn try_get(
        res: &sea_orm::QueryResult,
        pre: &str,
        col: &str,
    ) -> Result<Self, sea_orm::TryGetError> {
        if pre.is_empty() {
            Self::try_get_by(res, col)
        } else {
            Self::try_get_by(res, std::format!("{pre}{col}").as_str())
        }
    }

    fn try_get_by_index(
        res: &sea_orm::QueryResult,
        index: usize,
    ) -> Result<Self, sea_orm::TryGetError> {
        Self::try_get_by(res, index)
    }
}

impl ValueType for Race {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::Int(Some(value)) = v {
            <Race as std::convert::TryFrom<i32>>::try_from(value).map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(Race).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::Integer
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::Int
    }

    fn unwrap(v: Value) -> Self {
        <Self as sea_query::ValueType>::try_from(v).unwrap_or_default()
    }

    fn expect(v: Value, msg: &str) -> Self {
        <Self as sea_query::ValueType>::try_from(v).expect(msg)
    }

    fn enum_type_name() -> Option<&'static str> {
        None
    }
}
