use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(
    Component,
    TryFromPrimitive,
    IntoPrimitive,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Default,
    Reflect,
)]
#[repr(u32)]
pub enum BaseClass {
    Mystic,
    #[default]
    Fighter,
    MaleFighter,
    FemaleFighter,
}
impl BaseClass {
    pub const fn base_p_atk(&self) -> f32 {
        match self {
            BaseClass::Mystic => 3.0,
            BaseClass::Fighter => 4.0,
            _ => 4.0,
        }
    }

    pub const fn base_m_atk(&self) -> f32 {
        6.0
    }

    pub const fn base_critical_rate(&self) -> f32 {
        4.0
    }

    pub fn default_classes() -> Vec<BaseClass> {
        vec![BaseClass::Fighter, BaseClass::Mystic]
    }

    pub fn kamael_classes() -> Vec<BaseClass> {
        vec![BaseClass::MaleFighter, BaseClass::FemaleFighter]
    }
}
