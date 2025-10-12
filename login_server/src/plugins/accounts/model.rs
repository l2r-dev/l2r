use crate::utils::password::{PasswordError, hash_password, verify_password};
use bevy::prelude::*;
use chrono::{NaiveDateTime, Utc};
use l2r_core::{
    db::{DbRepository, PrimaryKeyColumns, UpdatableModel},
    model::access_level::AccessLevel,
};
use sea_orm::{
    self, ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter, entity::prelude::*,
};
use std::fmt;

pub type AccountsRepository = DbRepository<Uuid, Entity>;

#[derive(Clone, Component, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: Uuid,
    name: String,
    password: String,
    email: Option<String>,
    created_time: NaiveDateTime,
    access_level: AccessLevel,
    last_ip: Option<String>,
    last_server: i16,
}

impl PrimaryKeyColumns for Model {
    type Column = Column;

    fn pk_columns() -> &'static [Self::Column] {
        &[Column::Id]
    }
}

impl UpdatableModel for Model {
    type Column = Column;

    fn update_columns() -> &'static [Self::Column] {
        &[
            Column::Name,
            Column::Password,
            Column::Email,
            Column::AccessLevel,
            Column::LastIp,
            Column::LastServer,
        ]
    }
}

impl fmt::Debug for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Model {{ id: {}, name: {} }}", self.id, self.name)
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Model {
    pub fn new(name: String, password: &str) -> Result<Self, PasswordError> {
        let hashed_password = hash_password(password)?;
        Ok(Model {
            id: Uuid::now_v7(),
            name,
            password: hashed_password,
            email: None,
            created_time: Utc::now().naive_utc(),
            access_level: AccessLevel::default(),
            last_ip: None,
            last_server: 0,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    #[allow(dead_code)]
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn verify_password(&self, plain_password: &str) -> Result<bool, PasswordError> {
        verify_password(plain_password, &self.password)
    }

    #[allow(dead_code)]
    pub fn set_password(&mut self, plain_password: &str) -> Result<(), PasswordError> {
        self.password = hash_password(plain_password)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn created_time(&self) -> NaiveDateTime {
        self.created_time
    }

    pub fn access_level(&self) -> AccessLevel {
        self.access_level
    }

    #[allow(dead_code)]
    pub fn is_admin(&self) -> bool {
        self.access_level == AccessLevel::Admin
    }

    pub fn is_online(&self) -> bool {
        self.last_ip.is_some()
    }
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
