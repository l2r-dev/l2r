use super::{level::Level, level_exp_data::LEVEL_EXP_DATA};
use crate::stats::{DoubleStats, StatTrait, Stats};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

pub type LevelExp = (u32, u64);
pub type Exp = u64;
pub type Sp = u32;

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, PartialEq, Reflect, Serialize)]
#[require(super::ProgressRatesStats)]
#[serde(default)]
pub struct ProgressStats(DoubleStats<ProgressStat>);

impl ProgressStats {
    pub fn new(exp: Exp, sp: Sp) -> Self {
        let mut stats = ProgressStats::default();
        stats.set_exp(exp);
        stats.set_sp(sp);
        stats
    }

    pub fn exp(&self) -> Exp {
        self.get(&ProgressStat::Exp) as Exp
    }

    pub fn exp_percent(&self, level: Level) -> f64 {
        let exp_gained = self.exp_gained_on_current_level(level);
        let exp_to_next = self.exp_to_next_level(level) as f64;

        if exp_to_next <= 0.0 {
            0.0
        } else {
            exp_gained / exp_to_next
        }
    }

    pub fn sp(&self) -> Sp {
        self.get(&ProgressStat::Sp) as Sp
    }

    pub fn vitality_points(&self) -> u32 {
        self.get(&ProgressStat::VitalityPoints) as u32
    }

    pub fn set_exp(&mut self, exp: Exp) {
        self.insert(ProgressStat::Exp, exp as f64);
    }

    pub fn add_exp(&mut self, exp: Exp, exp_modifier: f64) {
        let modified_exp = (exp as f64 * exp_modifier).round() as u64;
        let new_exp = self.exp() + modified_exp;
        self.set_exp(new_exp);
    }

    pub fn set_sp(&mut self, sp: Sp) {
        self.insert(ProgressStat::Sp, sp as f64);
    }

    pub fn add_sp(&mut self, sp: Sp, sp_modifier: f64) {
        let modified_sp = (sp as f64 * sp_modifier).round() as Sp;
        let new_sp = self.sp() + modified_sp;
        self.insert(ProgressStat::Sp, new_sp as f64);
    }

    pub fn set_vitality_points(&mut self, points: u32) {
        self.insert(ProgressStat::VitalityPoints, points as f64);
    }

    pub fn exp_lost_percent(&self, level: Level) -> f64 {
        let level = u32::from(level);
        match level {
            1..=49 => 10.0 - (level - 1) as f64 * 0.125,
            50..=75 => 4.0,
            76 => 2.5,
            77 => 2.0,
            78 => 1.5,
            79..=85 => 1.0,
            _ => 0.0,
        }
    }

    pub fn exp_lost(&mut self, modifier: f64, level: Level) {
        let lost_exp = ((self.exp_to_next_level(level) as f64)
            * (self.exp_lost_percent(level) / 100.0))
            .round();
        let lost_exp = (lost_exp * modifier).round();
        let new_exp = self.exp() as f64 - lost_exp;
        self.insert(ProgressStat::Exp, new_exp);
    }

    pub fn get_exp_by_level(level: Level) -> Option<Exp> {
        let level: usize = level.into();
        if level > 0 && level <= LEVEL_EXP_DATA.len() {
            Some(LEVEL_EXP_DATA[level - 1].1)
        } else {
            None
        }
    }

    pub fn calculate_level_by_exp(&self) -> Level {
        let exp = self.exp();
        let idx = LEVEL_EXP_DATA.binary_search_by(|&(_, required_exp)| required_exp.cmp(&exp));

        match idx {
            Ok(i) => (LEVEL_EXP_DATA[i].0).into(),
            Err(i) => {
                if i == 0 {
                    1u32.into()
                } else {
                    (LEVEL_EXP_DATA[i - 1].0).into()
                }
            }
        }
    }

    pub fn exp_gained_on_current_level(&self, level: Level) -> f64 {
        if let Some(exp_for_current) = Self::get_exp_by_level(level) {
            (self.exp() as f64) - (exp_for_current as f64)
        } else {
            0.0
        }
    }

    pub fn exp_to_next_level(&self, current_level: Level) -> u64 {
        if let (Some(exp_for_current), Some(exp_for_next)) = (
            Self::get_exp_by_level(current_level),
            Self::get_exp_by_level(current_level + 1.into()),
        ) {
            exp_for_next - exp_for_current
        } else {
            0
        }
    }

    pub fn test_data() -> Self {
        let mut stats = ProgressStats::default();
        stats.set_exp(300000);
        stats.set_sp(1500);
        stats
    }
}

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize)]
pub enum ProgressStat {
    Exp,
    Sp,
    VitalityPoints,
}

impl StatTrait for ProgressStat {}
