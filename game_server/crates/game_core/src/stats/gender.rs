use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};

#[derive(
    Hash,
    Eq,
    Copy,
    IntoPrimitive,
    TryFromPrimitive,
    Debug,
    Clone,
    PartialEq,
    Component,
    Default,
    Serialize,
    Deserialize,
    Reflect,
)]
#[repr(u32)]
pub enum Gender {
    #[default]
    Male,
    Female,
    Etc,
}

impl TryFrom<i32> for Gender {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Gender::try_from_primitive(value as u32)
    }
}

impl From<Gender> for i32 {
    fn from(value: Gender) -> Self {
        value as i32
    }
}

impl From<&[u8]> for Gender {
    fn from(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        let value = i32::from_le_bytes(arr);
        <Gender as std::convert::TryFrom<i32>>::try_from(value).unwrap_or_default()
    }
}
