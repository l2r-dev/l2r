use crate::stats::{StatTrait, StatValue, Stats, UIntStats};
use bevy::{platform::collections::HashMap, prelude::*};
use l2r_core::model::{base_class::BaseClass, generic_number::GenericNumber};
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
use strum::{EnumIter, IntoEnumIterator};

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
pub struct Level(u32);
impl Level {
    pub fn diff(&self, other: &Self) -> u32 {
        self.0.abs_diff(other.0)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GenericNumber<u32> for Level {
    fn value(&self) -> u32 {
        self.0
    }
}

impl From<Level> for Value {
    fn from(level: Level) -> Self {
        Value::Int(Some(level.0 as i32))
    }
}

impl TryGetable for Level {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by(idx)?;
        Ok(Level(value as u32))
    }
}

impl ValueType for Level {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Int(Some(val)) => {
                if val >= 0 {
                    Ok(Level(val as u32))
                } else {
                    Err(ValueTypeErr)
                }
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Level).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::Integer
    }

    fn array_type() -> ArrayType {
        ArrayType::Int
    }
}

l2r_core::impl_std_math_operations!(Level, u32);
l2r_core::impl_primitive_conversions!(Level, u32);

#[derive(Clone, Component, Debug, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize)]
#[serde(default)]
pub struct ProgressLevelStats(UIntStats<ProgressLevelStat>);

impl ProgressLevelStats {
    pub fn new(level: Level) -> Self {
        let mut stats = HashMap::default();
        stats.insert(ProgressLevelStat::Level, level.into());
        stats.insert(ProgressLevelStat::PrevLevel, level.into());
        Self(stats.into())
    }

    pub fn level(&self) -> Level {
        self.get(&ProgressLevelStat::Level).into()
    }

    pub fn set_level(&mut self, level: Level) {
        let prev_level = self.level();
        if level != prev_level {
            self.insert(ProgressLevelStat::PrevLevel, prev_level.into());
            self.insert(ProgressLevelStat::Level, level.into());
        }
    }

    pub fn prev_level(&self) -> Level {
        self.get(&ProgressLevelStat::PrevLevel).into()
    }
}

impl Default for ProgressLevelStats {
    fn default() -> Self {
        let mut stats = HashMap::default();
        for stat in ProgressLevelStat::iter() {
            stats.insert(stat, stat.default_value::<u32>(BaseClass::default()));
        }
        Self(stats.into())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum ProgressLevelStat {
    Level,
    PrevLevel,
}

impl StatTrait for ProgressLevelStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        use ProgressLevelStat::*;
        let value = match self {
            Level => 1.0,
            PrevLevel => 1.0,
        };
        V::from(value).unwrap_or_default()
    }
}
