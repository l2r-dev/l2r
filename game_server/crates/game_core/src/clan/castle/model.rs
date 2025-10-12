use bevy::prelude::*;
use chrono::NaiveDateTime;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sea_orm::{
    self as sea_orm, ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter,
    entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::Display;

#[derive(
    Clone,
    Display,
    Component,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    Reflect,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
)]
#[repr(u8)]
pub enum CastleId {
    Gludio = 1,
    Dion = 2,
    Giran = 3,
    Oren = 4,
    Aden = 5,
    Innadril = 6,
    Goddard = 7,
    Rune = 8,
    Schuttgart = 9,
}

impl From<CastleId> for i32 {
    fn from(id: CastleId) -> i32 {
        id as i32
    }
}

#[derive(Clone, Component, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "castle")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub tax_percent: i32,
    pub treasury: i64,
    pub siege_date: NaiveDateTime,
    pub reg_time_over: bool,
    pub reg_time_end: NaiveDateTime,
    pub show_npc_crest: bool,
    pub ticket_buy_count: i16,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct CastleConfig {
    pub tax_percent: i32,
    pub treasury: i64,
    pub siege_date: NaiveDateTime,
    pub reg_time_over: bool,
    pub reg_time_end: NaiveDateTime,
    pub show_npc_crest: bool,
    pub ticket_buy_count: i16,
}

impl Model {
    pub fn new(id: CastleId, config: CastleConfig) -> Self {
        Self {
            id: id.into(),
            name: id.to_string(),
            tax_percent: config.tax_percent,
            treasury: config.treasury,
            siege_date: config.siege_date,
            reg_time_over: config.reg_time_over,
            reg_time_end: config.reg_time_end,
            show_npc_crest: config.show_npc_crest,
            ticket_buy_count: config.ticket_buy_count,
        }
    }
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
