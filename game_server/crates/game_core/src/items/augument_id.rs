use crate::items::Id;
use bevy::prelude::*;
use l2r_core::model::generic_number::GenericNumber;
use sea_orm::{
    TryFromU64, TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
    str::FromStr,
};

#[derive(
    Clone,
    Copy,
    Default,
    Debug,
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
pub struct EnchantOptions([AugumentId; 3]);
impl EnchantOptions {
    pub fn to_le_bytes(self) -> [u8; 6] {
        let mut buffer = [0u8; 6];
        for (i, enchant_option) in self.0.iter().enumerate() {
            let u16_option: u16 = (*enchant_option).into();
            let bytes = u16_option.to_le_bytes();
            let start_pos = i * 2;
            let end_pos = (i * 2) + 2;
            buffer[start_pos..end_pos].copy_from_slice(&bytes);
        }
        buffer
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deref,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Reflect,
    Default,
)]
pub struct AugumentId(u32);

impl From<Id> for AugumentId {
    fn from(id: Id) -> Self {
        AugumentId(id.into())
    }
}

impl From<AugumentId> for Id {
    fn from(aug_id: AugumentId) -> Self {
        Id::from(aug_id.0)
    }
}

impl fmt::Display for AugumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GenericNumber<u32> for AugumentId {
    fn value(&self) -> u32 {
        self.0
    }
}

impl From<AugumentId> for Value {
    fn from(id: AugumentId) -> Self {
        Value::Int(Some(id.0 as i32))
    }
}

impl TryGetable for AugumentId {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by_nullable(idx)?;
        Ok(AugumentId(value as u32))
    }
}

impl ValueType for AugumentId {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Int(Some(val)) => {
                if val >= 0 {
                    Ok(AugumentId(val as u32))
                } else {
                    Err(ValueTypeErr)
                }
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(AugumentId).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::Integer
    }

    fn array_type() -> ArrayType {
        ArrayType::Int
    }
}

impl TryFromU64 for AugumentId {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
        if n > u32::MAX as u64 {
            return Err(sea_orm::DbErr::Type(format!(
                "AugumentId value cannot be greater than {}: {}",
                u32::MAX,
                n
            )));
        }
        Ok(AugumentId(n as u32))
    }
}

impl Nullable for AugumentId {
    fn null() -> Value {
        Value::Int(None)
    }
}

l2r_core::impl_std_math_operations!(AugumentId, u32);
l2r_core::impl_primitive_conversions!(AugumentId, u32);
