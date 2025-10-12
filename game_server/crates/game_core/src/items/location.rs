use crate::items::DollSlot;
use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ColumnType, ValueType, ValueTypeErr},
};
use serde::Deserialize;
use strum::{Display, EnumDiscriminants, EnumIter};

#[derive(Clone, Copy, Debug, Deserialize, EnumDiscriminants, PartialEq, Reflect)]
#[strum_discriminants(name(ItemLocationVariant))]
#[strum_discriminants(derive(
    Display,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
    Default,
    Reflect
))]
#[repr(i16)]
pub enum ItemLocation {
    Unknown = -1,
    ClanWarehouse,
    Freight,
    Inventory,
    Lease,
    Mail,
    PaperDoll(DollSlot),
    Pet,
    PetEquip,
    Refund,
    Warehouse,
    Store,
    #[strum_discriminants(default)]
    World(Vec3),
}

impl ItemLocation {
    pub fn location_data(&self) -> u32 {
        match self {
            ItemLocation::PaperDoll(slot) => (*slot).into(),
            _ => 0,
        }
    }
}

impl From<ItemLocationVariant> for Value {
    fn from(location: ItemLocationVariant) -> Self {
        Value::SmallInt(Some(location.into()))
    }
}

impl TryGetable for ItemLocationVariant {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i16 = res.try_get_by(idx)?;
        ItemLocationVariant::try_from_primitive(value).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to convert {value} to ItemLocation"
            )))
        })
    }
}

impl ValueType for ItemLocationVariant {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::SmallInt(Some(val)) = v {
            ItemLocationVariant::try_from_primitive(val).map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(ItemLocationVariant).to_string()
    }

    fn column_type() -> ColumnType {
        ColumnType::SmallInteger
    }

    fn array_type() -> ArrayType {
        ArrayType::SmallInt
    }
}
