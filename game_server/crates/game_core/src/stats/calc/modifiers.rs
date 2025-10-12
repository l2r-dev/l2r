use super::StatsOperation;
use crate::stats::StatKind;
use bevy::{platform::collections::HashMap, prelude::*};

pub struct StatModifiersComponentsPlugin;
impl Plugin for StatModifiersComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StatModifiers>()
            .register_type::<StatModifier>();
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct StatModifier {
    pub stat: StatKind,
    pub operation: StatsOperation<f32>,
    pub priority: i32, // For controlling application order
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct StatModifiers(HashMap<String, StatModifier>);

impl StatModifiers {
    pub fn add_modifier(&mut self, source: String, modifier: StatModifier) {
        self.0.insert(source, modifier);
    }

    pub fn remove_modifier(&mut self, source: &str) {
        self.0.remove(source);
    }

    pub fn remove_modifier_contains(&mut self, source: &str) {
        self.0.retain(|key, _| !key.contains(source));
    }

    // Usually set must be only one, but if we have multiple somehow, we need to get the strongest one
    pub fn get_top_set_modifier(&self, stat: StatKind) -> Option<f32> {
        self.0
            .values()
            .filter(|modifier| {
                modifier.stat == stat && matches!(modifier.operation, StatsOperation::Set(_))
            })
            .max_by_key(|modifier| match modifier.operation {
                StatsOperation::Set(value) => value as i32,
                _ => 0,
            })
            .map(|modifier| match modifier.operation {
                StatsOperation::Set(value) => value,
                _ => 0.0,
            })
    }

    pub fn apply_to_stat(&self, stat: StatKind, base_value: f32) -> f32 {
        let mut value = base_value;

        // Apply modifiers by priority (ascending order, 0 is highest priority)
        // Set modifiers are applied in get_top_set_modifier, before other modifiers
        let mut modifiers: Vec<&StatModifier> = self
            .0
            .values()
            .filter(|modifier| {
                modifier.stat == stat && !matches!(modifier.operation, StatsOperation::Set(_))
            })
            .collect();

        modifiers.sort_by_key(|modifier| modifier.priority);

        for modifier in modifiers {
            value = modifier.operation.apply(value);
        }

        value
    }

    pub fn merge(&mut self, other: &StatModifiers) {
        for (source, modifier) in &other.0 {
            self.add_modifier(source.clone(), modifier.clone());
        }
    }

    pub fn unmerge(&mut self, other: &StatModifiers) {
        for source in other.0.keys() {
            self.0.remove(source);
        }
    }
}
