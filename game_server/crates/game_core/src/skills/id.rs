use bevy::prelude::*;
use l2r_core::model::generic_number::GenericNumber;
use scripting::{bindings::InteropError, prelude::ScriptValue};
use sea_orm::{
    ColumnType, TryFromU64, TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ValueType, ValueTypeErr},
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
impl Id {
    pub fn from_skill_script(s: ScriptValue) -> Result<Self, InteropError> {
        match s.clone() {
            ScriptValue::Map(skill_map) => {
                if let Some(id_value) = skill_map.get("id") {
                    if let ScriptValue::Integer(id) = id_value {
                        Ok(Id::from(*id))
                    } else {
                        Err(InteropError::invalid_index(
                            id_value.clone(),
                            "Invalid skill ID".to_owned(),
                        ))
                    }
                } else {
                    Err(InteropError::invalid_index(
                        ScriptValue::Map(skill_map.clone()),
                        "Skill ID not found".to_owned(),
                    ))
                }
            }
            _ => Err(InteropError::invalid_index(s, "Invalid skill".to_owned())),
        }
    }
}

impl From<Id> for ScriptValue {
    fn from(id: Id) -> Self {
        Self::Integer(id.0 as i64)
    }
}

impl From<ScriptValue> for Id {
    fn from(value: ScriptValue) -> Self {
        match value {
            ScriptValue::Integer(id) => Self(id as u32),
            _ => Self(0),
        }
    }
}

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

impl TryFromU64 for Id {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
        if n > u32::MAX as u64 {
            return Err(sea_orm::DbErr::Type(format!(
                "Skill Id value cannot be greater than {}: {}",
                u32::MAX,
                n
            )));
        }
        Ok(Id(n as u32))
    }
}

l2r_core::impl_std_math_operations!(Id, u32);
l2r_core::impl_primitive_conversions!(Id, u32);

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::platform::collections::HashMap;

    #[test]
    fn test_from_skill_script() {
        let mut skill_map = HashMap::new();
        skill_map.insert("id".to_string(), ScriptValue::Integer(1));
        let script_value = ScriptValue::Map(skill_map);

        let result = Id::from_skill_script(script_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Id(1));
    }

    #[test]
    fn test_from_skill_script_invalid_id() {
        let mut skill_map = HashMap::new();
        skill_map.insert("id".to_string(), ScriptValue::String("1".into()));
        let script_value = ScriptValue::Map(skill_map);

        let result = Id::from_skill_script(script_value);
        assert!(result.is_err());
    }
}
