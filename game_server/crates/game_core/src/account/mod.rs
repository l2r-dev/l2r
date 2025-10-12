use bevy::prelude::*;
use l2r_core::{
    crypt::session_keys::{SessionAccount, SessionKey},
    model::access_level::AccessLevel,
};
use sea_orm::prelude::Uuid;
use std::fmt;

#[derive(Clone, Component, Eq, PartialEq)]
pub struct Account {
    id: Uuid,
    name: String,
    access: AccessLevel,
    key: SessionKey,
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Account {{ id: {}, name: {} }}", self.id, self.name)
    }
}

impl Account {
    pub fn new(name: String, session_account: SessionAccount) -> Self {
        Self {
            name,
            id: session_account.id,
            access: session_account.access,
            key: session_account.key,
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn access(&self) -> AccessLevel {
        self.access
    }
    pub fn key(&self) -> &SessionKey {
        &self.key
    }
}
