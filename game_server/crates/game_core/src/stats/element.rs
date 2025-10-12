use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[repr(u16)]
pub enum Element {
    Fire,
    Water,
    Wind,
    Earth,
    Holy,
    Dark,
}

#[derive(
    Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, FromJsonQueryResult, Reflect,
)]
pub struct ItemElementsInfo {
    pub attack_element: Option<(Element, u16)>,
    pub defence_elements: Option<[u16; 6]>,
}

impl ItemElementsInfo {
    pub fn to_le_bytes(&self) -> [u8; 16] {
        let (attack_elem, attack_val) = self
            .attack_element
            .map(|(elem, val)| (u16::from(elem), val))
            .unwrap_or_default();

        let mut buffer = [0u8; 16];

        buffer[..4]
            .copy_from_slice(&[attack_elem.to_le_bytes(), attack_val.to_le_bytes()].concat());

        if let Some(ref elements) = self.defence_elements {
            for (i, &value) in elements.iter().enumerate() {
                let start_pos = 4 + i * 2; // 4 (2 for attack_elem + 2 for attack_val) + 2 for each defence element
                let end_pos = start_pos + 2;
                buffer[start_pos..end_pos].copy_from_slice(&value.to_le_bytes());
            }
        }

        buffer
    }
}
