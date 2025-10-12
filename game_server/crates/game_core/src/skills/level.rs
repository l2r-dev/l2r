use bevy::prelude::*;
use derive_more::{From, Into};
use scripting::{bindings::InteropError, prelude::ScriptValue};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

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
    From,
    Into,
)]
pub struct Level(u32);
impl Level {
    pub fn from_skill_script(s: ScriptValue) -> Result<Self, InteropError> {
        match s.clone() {
            ScriptValue::Map(skill_map) => {
                if let Some(id_value) = skill_map.get("level") {
                    if let ScriptValue::Integer(level) = id_value {
                        Ok(Level::from(*level))
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

impl From<Level> for ScriptValue {
    fn from(level: Level) -> Self {
        Self::Integer(level.0 as i64)
    }
}

impl From<ScriptValue> for Level {
    fn from(value: ScriptValue) -> Self {
        match value {
            ScriptValue::Integer(level) => Self(level as u32),
            _ => Self(0),
        }
    }
}

impl From<usize> for Level {
    fn from(level: usize) -> Self {
        Self(level as u32)
    }
}

impl From<Level> for usize {
    fn from(level: Level) -> Self {
        level.0 as usize
    }
}

impl From<i64> for Level {
    fn from(level: i64) -> Self {
        Self(level as u32)
    }
}

impl From<Level> for u16 {
    fn from(level: Level) -> Self {
        level.0 as u16
    }
}

impl FromStr for Level {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Self)
    }
}

impl From<i32> for Level {
    fn from(level: i32) -> Self {
        Self(level as u32)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = f.width().unwrap_or(0);
        write!(f, "{:0width$}", self.0, width = width)
    }
}

impl std::ops::Mul for Level {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Div for Level {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl From<Level> for i32 {
    fn from(level: Level) -> Self {
        level.0 as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::platform::collections::HashMap;

    #[test]
    fn test_from_skill_script() {
        let mut skill_map = HashMap::new();
        skill_map.insert("level".to_string(), ScriptValue::Integer(1));
        let script_value = ScriptValue::Map(skill_map);

        let result = Level::from_skill_script(script_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Level(1));
    }

    #[test]
    fn test_from_skill_script_invalid_id() {
        let mut skill_map = HashMap::new();
        skill_map.insert("level".to_string(), ScriptValue::String("1".into()));
        let script_value = ScriptValue::Map(skill_map);

        let result = Level::from_skill_script(script_value);
        assert!(result.is_err());
    }
}
