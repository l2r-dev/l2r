use bevy::prelude::*;
use l2r_core::model::generic_number::GenericNumber;
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ColumnType, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
    str::FromStr,
};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deref,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Reflect,
)]
pub struct Id(u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GenericNumber<u32> for Id {
    fn value(&self) -> u32 {
        self.0
    }
}

impl From<Id> for Value {
    fn from(id: Id) -> Self {
        Value::Int(Some(id.0 as i32))
    }
}

impl TryGetable for Id {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by(idx)?;
        Ok(Id(value as u32))
    }
}

impl ValueType for Id {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Int(Some(val)) => {
                if val >= 0 {
                    Ok(Id(val as u32))
                } else {
                    Err(ValueTypeErr)
                }
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Id).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::Integer
    }

    fn array_type() -> ArrayType {
        ArrayType::Int
    }
}

l2r_core::impl_std_math_operations!(Id, u32);
l2r_core::impl_primitive_conversions!(Id, u32);
