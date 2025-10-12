use bevy::prelude::*;
use chrono::{NaiveDateTime, Utc};
use derive_more::{From, Into};
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};

/// A reflection-friendly wrapper around chrono::NaiveDateTime
///
/// This newtype allows us to use NaiveDateTime in Bevy contexts that require Reflect,
/// such as scripting and dynamic scenes. The wrapper implements Reflect as an opaque type,
/// meaning it can be serialized/deserialized but its internal structure is not exposed
/// for field-level reflection.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Reflect,
    Serialize,
    Deserialize,
    Deref,
    DerefMut,
    From,
    Into,
)]
#[reflect(opaque)]
#[reflect(Default, PartialEq, Serialize, Deserialize)]
pub struct ReflectableDateTime(NaiveDateTime);

impl ReflectableDateTime {
    /// Create a new ReflectableDateTime from a NaiveDateTime
    pub fn new(datetime: NaiveDateTime) -> Self {
        Self(datetime)
    }

    /// Get the current UTC time as a ReflectableDateTime
    pub fn now() -> Self {
        Self(Utc::now().naive_utc())
    }
}

impl AsRef<NaiveDateTime> for ReflectableDateTime {
    fn as_ref(&self) -> &NaiveDateTime {
        &self.0
    }
}

// Display and formatting traits
impl std::fmt::Display for ReflectableDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ReflectableDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

// SeaORM trait implementations for database storage
impl From<ReflectableDateTime> for Value {
    fn from(datetime: ReflectableDateTime) -> Self {
        Value::ChronoDateTime(Some(Box::new(datetime.0)))
    }
}

impl TryGetable for ReflectableDateTime {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: NaiveDateTime = res.try_get_by(idx)?;
        Ok(ReflectableDateTime::from(value))
    }
}

impl ValueType for ReflectableDateTime {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::ChronoDateTime(Some(datetime)) => Ok(ReflectableDateTime::from(*datetime)),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(ReflectableDateTime).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::TimestampWithTimeZone
    }

    fn array_type() -> ArrayType {
        ArrayType::TimeDateTimeWithTimeZone
    }
}

impl Nullable for ReflectableDateTime {
    fn null() -> Value {
        Value::ChronoDateTimeUtc(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflectable_datetime_creation() {
        let now = chrono::DateTime::from_timestamp(1000000000, 0)
            .unwrap()
            .naive_utc();
        let wrapper = ReflectableDateTime::new(now);
        assert_eq!(*wrapper, now); // Using Deref
    }

    #[test]
    fn test_conversion_traits() {
        let now = chrono::DateTime::from_timestamp(1000000000, 0)
            .unwrap()
            .naive_utc();
        let wrapper = ReflectableDateTime::from(now);
        let back: NaiveDateTime = wrapper.into();
        assert_eq!(back, now);
    }

    #[test]
    fn test_default() {
        let default_wrapper = ReflectableDateTime::default();
        let expected = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();
        assert_eq!(*default_wrapper, expected); // Using Deref
    }
}
