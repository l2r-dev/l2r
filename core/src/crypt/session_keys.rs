use super::blowfish::BlowfishKey;
use crate::model::access_level::AccessLevel;
use bevy::prelude::*;
use redis_macros::{FromRedisValue, ToRedisArgs};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Component, Copy, Default, Deref, Deserialize, Eq, PartialEq, Serialize, Reflect,
)]
pub struct SessionKey(BlowfishKey);

impl SessionKey {
    pub fn new() -> Self {
        Self(BlowfishKey::new())
    }

    pub fn blowfish_key(&self) -> &BlowfishKey {
        &self.0
    }

    pub fn get_login(&self) -> i64 {
        let bytes = self.0.to_le_bytes();
        i64::from_le_bytes(bytes[0..8].try_into().unwrap_or_default())
    }

    pub fn get_game(&self) -> i64 {
        let bytes = self.0.to_le_bytes();
        i64::from_le_bytes(bytes[8..16].try_into().unwrap_or_default())
    }

    pub fn validate(&self, key: SessionKey) -> bool {
        self == &key
    }
}

impl From<[u8; 16]> for SessionKey {
    fn from(bytes: [u8; 16]) -> Self {
        Self(BlowfishKey::from(bytes))
    }
}

#[derive(
    Clone,
    Component,
    Copy,
    Default,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
    FromRedisValue,
    ToRedisArgs,
)]
pub struct SessionAccount {
    pub id: Uuid,
    pub access: AccessLevel,
    pub key: SessionKey,
}
