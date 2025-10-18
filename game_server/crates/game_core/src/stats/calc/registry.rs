use super::StatModifiers;
use crate::stats::*;
use bevy::platform::collections::HashMap;

#[derive(Default, Resource)]
pub struct StatFormulaRegistry(
    HashMap<StatKind, Box<dyn Fn(FormulaArguments) -> f32 + Send + Sync + 'static>>,
);

#[derive(Clone)]
pub struct FormulaArguments<'a> {
    pub base_value: f32,
    pub primal: &'a PrimalStats,
    pub stat_modifiers: &'a StatModifiers,
    pub level: Level,
    pub is_character: bool,
    pub is_pet: bool,
}

impl<'a> FormulaArguments<'a> {
    pub fn new(
        base_value: f32,
        primal: &'a PrimalStats,
        stat_modifiers: &'a StatModifiers,
        level: Level,
        is_character: bool,
        is_pet: bool,
    ) -> Self {
        Self {
            base_value,
            primal,
            stat_modifiers,
            level,
            is_character,
            is_pet,
        }
    }

    pub fn from_params(base_value: f32, params: &'a StatsCalculateParams<'a>) -> Self {
        Self {
            base_value,
            primal: params.primal_stats(),
            stat_modifiers: params.stat_modifiers(),
            level: params.progress_level().level(),
            is_character: params.is_character(),
            is_pet: params.is_pet(),
        }
    }
}

impl StatFormulaRegistry {
    pub fn level_modifier(level: Level) -> f32 {
        let level: f32 = level.into();
        (level + 89.0) / 100.0
    }

    pub fn register_formula<F>(&mut self, stat: StatKind, formula: F) -> &mut Self
    where
        F: Fn(FormulaArguments) -> f32 + Send + Sync + 'static,
    {
        self.0.insert(stat, Box::new(formula));
        self
    }

    fn calculate_base<S: StatTrait>(&self, stat: S, formula_arguments: FormulaArguments) -> f32 {
        let stat_kind: StatKind = stat.into();
        if let Some(formula) = self.0.get(&stat_kind) {
            formula(formula_arguments)
        } else {
            formula_arguments.base_value
        }
    }

    pub fn calculate_final_value<S: StatTrait>(
        &self,
        stat: S,
        formula_arguments: FormulaArguments,
    ) -> f32 {
        let base_value = self.calculate_base(stat, formula_arguments.clone());

        formula_arguments
            .stat_modifiers
            .apply_to_stat(stat.into(), base_value)
    }
}
