use crate::stats::ClassId;
use bevy::prelude::*;
use num_enum::TryFromPrimitive;
use sea_orm::{
    Iden, IdenStatic, PrimaryKeyTrait, TryFromU64, TryGetError, TryGetable, Value,
    sea_query::{self, ValueType, ValueTypeErr},
};
use std::fmt;
use strum::{Display, EnumDiscriminants, EnumIter};

#[repr(i16)]
#[derive(Clone, Component, Copy, Debug, EnumDiscriminants, PartialEq, Reflect)]
#[strum_discriminants(name(SubClassVariant))]
#[strum_discriminants(derive(Display, EnumIter, TryFromPrimitive, Default, Reflect))]
pub enum SubClass {
    #[strum_discriminants(default)]
    Main(ClassId),
    SubClass1(ClassId),
    SubClass2(ClassId),
    SubClass3(ClassId),
}

impl SubClass {
    pub fn class_id(&self) -> ClassId {
        match self {
            SubClass::Main(class_id) => *class_id,
            SubClass::SubClass1(class_id) => *class_id,
            SubClass::SubClass2(class_id) => *class_id,
            SubClass::SubClass3(class_id) => *class_id,
        }
    }

    pub fn variant(&self) -> SubClassVariant {
        SubClassVariant::from(*self)
    }
}

impl From<(SubClassVariant, ClassId)> for SubClass {
    fn from((sub_class_variant, class_id): (SubClassVariant, ClassId)) -> Self {
        match sub_class_variant {
            SubClassVariant::Main => SubClass::Main(class_id),
            SubClassVariant::SubClass1 => SubClass::SubClass1(class_id),
            SubClassVariant::SubClass2 => SubClass::SubClass2(class_id),
            SubClassVariant::SubClass3 => SubClass::SubClass3(class_id),
        }
    }
}

impl From<SubClassVariant> for Value {
    fn from(class_id: SubClassVariant) -> Self {
        Value::SmallInt(Some(class_id as i16))
    }
}

impl TryGetable for SubClassVariant {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i16 = res.try_get_by(idx)?;
        <SubClassVariant as std::convert::TryFrom<i16>>::try_from(value).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to convert {value} to SubClassVariant enum"
            )))
        })
    }
}

impl ValueType for SubClassVariant {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::SmallInt(Some(value)) = v {
            <SubClassVariant as std::convert::TryFrom<i16>>::try_from(value)
                .map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(SubClassVariant).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::SmallInteger
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::SmallInt
    }
}

impl PrimaryKeyTrait for SubClassVariant {
    type ValueType = i16;

    fn auto_increment() -> bool {
        false
    }
}

impl Iden for SubClassVariant {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        write!(s, "{}", self.as_str()).unwrap();
    }
}

impl IdenStatic for SubClassVariant {
    fn as_str(&self) -> &str {
        match self {
            SubClassVariant::Main => "main",
            SubClassVariant::SubClass1 => "sub_class_1",
            SubClassVariant::SubClass2 => "sub_class_2",
            SubClassVariant::SubClass3 => "sub_class_3",
        }
    }
}

impl TryFromU64 for SubClassVariant {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
        let value = n.try_into().map_err(|_| {
            sea_orm::DbErr::Type(format!("Failed to convert {n} to SubClassVariant enum"))
        })?;

        <SubClassVariant as std::convert::TryFrom<i16>>::try_from(value).map_err(|_| {
            sea_orm::DbErr::Type(format!("Failed to convert {value} to SubClassVariant enum"))
        })
    }
}
