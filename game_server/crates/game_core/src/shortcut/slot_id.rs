use bevy::prelude::*;
use derive_more::{From, Into};
use sea_orm::{
    TryFromU64, TryGetable, Value,
    sea_query::{self, ValueType, ValueTypeErr},
};

#[derive(Clone, Copy, Debug, From, Into, PartialEq, Reflect)]
pub struct SlotId(u32);
impl SlotId {
    pub const SLOTS: u32 = 12;

    pub fn new(slot: u32, page: u32) -> Self {
        Self(slot + page * Self::SLOTS)
    }

    pub fn slot(&self) -> u32 {
        self.0 % Self::SLOTS
    }

    pub fn page(&self) -> u32 {
        self.0 / Self::SLOTS
    }
}

impl From<SlotId> for Value {
    fn from(slot_id: SlotId) -> Self {
        Value::Int(Some(slot_id.0 as i32))
    }
}

impl TryGetable for SlotId {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let value: i32 = res.try_get_by(idx)?;
        if value < 0 {
            return Err(sea_orm::TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "SlotId value cannot be negative: {value}"
            ))));
        }
        Ok(SlotId(value as u32))
    }
}

impl ValueType for SlotId {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::Int(Some(value)) = v {
            if value < 0 {
                return Err(ValueTypeErr);
            }
            Ok(SlotId(value as u32))
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(SlotId).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::Integer
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::Int
    }
}

impl TryFromU64 for SlotId {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
        if n > u32::MAX as u64 {
            return Err(sea_orm::DbErr::Type(format!(
                "SlotId value cannot be greater than {}: {}",
                u32::MAX,
                n
            )));
        }
        Ok(SlotId(n as u32))
    }
}
