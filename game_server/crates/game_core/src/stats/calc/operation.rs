use crate::stats::StatValue;
use bevy::{log, prelude::*};
use num_traits::cast::NumCast;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Display, Reflect, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatsOperation<V: StatValue> {
    Set(V),
    Add(V),
    Sub(V),
    Mul(V),
    Div(V),
}

impl<V> StatsOperation<V>
where
    V: StatValue,
{
    pub fn apply(&self, base_value: V) -> V {
        match self {
            StatsOperation::Set(v) => *v,
            StatsOperation::Add(v) => base_value + *v,
            StatsOperation::Sub(v) => base_value - *v,
            StatsOperation::Mul(v) => base_value * *v,
            StatsOperation::Div(v) => {
                if *v != V::zero() {
                    base_value / *v
                } else {
                    log::warn!("Division by zero in stat operation: {}", self);
                    base_value
                }
            }
        }
    }

    pub fn convert<T>(&self) -> Result<StatsOperation<T>, BevyError>
    where
        T: StatValue,
    {
        let convert_value = |v: V| -> Result<T, BevyError> {
            NumCast::from(v).ok_or_else(|| {
                BevyError::from(format!(
                    "StatsOperation: Failed to convert {} to target type",
                    std::any::type_name::<V>()
                ))
            })
        };

        match self {
            StatsOperation::Set(v) => convert_value(*v).map(StatsOperation::Set),
            StatsOperation::Add(v) => convert_value(*v).map(StatsOperation::Add),
            StatsOperation::Sub(v) => convert_value(*v).map(StatsOperation::Sub),
            StatsOperation::Mul(v) => convert_value(*v).map(StatsOperation::Mul),
            StatsOperation::Div(v) => convert_value(*v).map(StatsOperation::Div),
        }
    }
}
