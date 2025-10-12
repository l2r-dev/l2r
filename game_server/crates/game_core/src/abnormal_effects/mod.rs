use crate::{
    skills::{Id, Skill},
    stats::StatsOperation,
};
use bevy::{
    log,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;

mod kind;
mod timers;

pub use kind::*;
pub use timers::*;

pub struct AbnormalEffectsComponentsPlugin;
impl Plugin for AbnormalEffectsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AbnormalEffect>()
            .register_type::<AbnormalEffects>()
            .register_type::<AbnormalEffectsChangeTracker>()
            .register_type::<AbnormalEffectsTimers>()
            .register_type::<AbnormalEffectTimer>()
            .register_type::<AbnormalKind>()
            .register_type::<RhythmKind>()
            .register_type::<BuffKind>()
            .register_type::<DebuffKind>()
            .register_type::<EffectOverTime>()
            .register_type::<StatsOperation<f32>>();
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct AbnormalEffect {
    skill: Skill,
    active: bool,
    kind: AbnormalKind,
}

impl AbnormalEffect {
    pub fn new(skill: Skill, kind: AbnormalKind, active: bool) -> Self {
        Self {
            skill,
            kind,
            active,
        }
    }

    pub fn skill(&self) -> &Skill {
        &self.skill
    }

    pub fn kind(&self) -> AbnormalKind {
        self.kind
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl From<&AbnormalEffect> for Skill {
    fn from(effect: &AbnormalEffect) -> Self {
        effect.skill
    }
}

impl AsRef<Skill> for AbnormalEffect {
    fn as_ref(&self) -> &Skill {
        &self.skill
    }
}

impl AsRef<AbnormalKind> for AbnormalEffect {
    fn as_ref(&self) -> &AbnormalKind {
        &self.kind
    }
}

impl Hash for AbnormalEffect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.skill.hash(state);
    }
}

impl PartialEq for AbnormalEffect {
    fn eq(&self, other: &Self) -> bool {
        self.skill.id() == other.skill.id()
    }
}
impl Eq for AbnormalEffect {}

#[derive(Clone, Component, Debug, Reflect)]
#[reflect(Component)]
#[require(AbnormalEffectsTimers, AbnormalEffectsChangeTracker)]
pub struct AbnormalEffects {
    effects: HashMap<AbnormalKindCategory, Vec<AbnormalEffect>>,
    max_buffs: usize,
    max_debuffs: usize,
    max_rhythm: usize,
}

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AbnormalEffectsChangeTracker {
    previous: HashSet<AbnormalEffect>,
    added: Vec<AbnormalEffect>,
    removed: Vec<AbnormalEffect>,
}

impl AbnormalEffectsChangeTracker {
    pub fn added(&self) -> &[AbnormalEffect] {
        &self.added
    }

    pub fn removed(&self) -> &[AbnormalEffect] {
        &self.removed
    }

    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }

    pub fn update(&mut self, current: &HashMap<AbnormalKindCategory, Vec<AbnormalEffect>>) {
        let current_set: HashSet<AbnormalEffect> = current
            .values()
            .flat_map(|vec| vec.iter())
            .copied()
            .collect();
        self.added = current_set.difference(&self.previous).copied().collect();
        self.removed = self.previous.difference(&current_set).copied().collect();
        self.previous = current_set;
    }
}

impl Default for AbnormalEffects {
    fn default() -> Self {
        let mut effects = HashMap::new();
        for variant in AbnormalKindCategory::iter() {
            effects.insert(variant, Vec::with_capacity(12));
        }
        Self {
            effects,
            max_buffs: 20,
            max_debuffs: 14,
            max_rhythm: 14,
        }
    }
}

impl AbnormalEffects {
    pub fn add(&mut self, new_effect: AbnormalEffect) {
        let new_kind = new_effect.kind;
        let category = new_kind.category();
        let effects_vec = self.get_vec_by_kind_mut(&category);

        // Find existing effect with same skill
        if let Some(existing_idx) = effects_vec
            .iter()
            .position(|e| e.skill.id() == new_effect.skill.id())
        {
            effects_vec.push(new_effect);
            effects_vec.remove(existing_idx);
        } else {
            effects_vec.push(new_effect);
        }

        self.remove_excess_effects(&category);
    }

    pub fn get(&self) -> Vec<&AbnormalEffect> {
        self.effects.values().flat_map(|vec| vec.iter()).collect()
    }

    pub fn effects(&self) -> &HashMap<AbnormalKindCategory, Vec<AbnormalEffect>> {
        &self.effects
    }

    pub fn has_effect(&self, skill_id: Id) -> bool {
        self.effects
            .values()
            .flat_map(|vec| vec.iter())
            .any(|effect| effect.skill.id() == skill_id)
    }

    pub fn remove(&mut self, skill_id: Id) -> Option<AbnormalEffect> {
        for effects_vec in self.effects.values_mut() {
            if let Some(pos) = effects_vec.iter().position(|e| e.skill.id() == skill_id) {
                return Some(effects_vec.remove(pos));
            }
        }
        None
    }

    pub fn remove_by_skill_id(&mut self, skill_id: Id) -> bool {
        self.remove(skill_id).is_some()
    }

    pub fn remove_all(&mut self) {
        for effects_vec in self.effects.values_mut() {
            effects_vec.clear();
        }
    }

    pub fn active_effects(&self) -> impl Iterator<Item = &AbnormalEffect> {
        self.effects
            .values()
            .flat_map(|vec| vec.iter())
            .filter(|effect| effect.active)
    }

    pub fn activate_effects_by_kind(&mut self, effect_kinds: Vec<AbnormalKind>) {
        for kind in effect_kinds {
            self.activate_effect_by_kind(&kind);
        }
    }

    pub fn set_max_buffs(&mut self, max_buffs: usize) {
        self.max_buffs = max_buffs;
    }

    pub fn set_max_debuffs(&mut self, max_debuffs: usize) {
        self.max_debuffs = max_debuffs;
    }

    pub fn set_max_rhythm(&mut self, max_rhythm: usize) {
        self.max_rhythm = max_rhythm;
    }

    pub fn max_buffs(&self) -> usize {
        self.max_buffs
    }

    pub fn max_debuffs(&self) -> usize {
        self.max_debuffs
    }

    pub fn max_rhythm(&self) -> usize {
        self.max_rhythm
    }

    pub fn buffs_len(&self) -> usize {
        self.effects[&AbnormalKindCategory::Buff]
            .iter()
            .filter(|effect| effect.active)
            .count()
    }

    pub fn debuff_len(&self) -> usize {
        self.effects[&AbnormalKindCategory::Debuff]
            .iter()
            .filter(|effect| effect.active)
            .count()
    }

    pub fn rhythm_len(&self) -> usize {
        self.effects[&AbnormalKindCategory::Rhythm]
            .iter()
            .filter(|effect| effect.active)
            .count()
    }

    pub fn buffs(&self) -> &Vec<AbnormalEffect> {
        &self.effects[&AbnormalKindCategory::Buff]
    }

    pub fn debuffs(&self) -> &Vec<AbnormalEffect> {
        &self.effects[&AbnormalKindCategory::Debuff]
    }

    pub fn rhythms(&self) -> &Vec<AbnormalEffect> {
        &self.effects[&AbnormalKindCategory::Rhythm]
    }

    pub fn buffs_mut(&mut self) -> &mut Vec<AbnormalEffect> {
        self.effects.get_mut(&AbnormalKindCategory::Buff).unwrap()
    }

    pub fn debuffs_mut(&mut self) -> &mut Vec<AbnormalEffect> {
        self.effects.get_mut(&AbnormalKindCategory::Debuff).unwrap()
    }

    pub fn rhythms_mut(&mut self) -> &mut Vec<AbnormalEffect> {
        self.effects.get_mut(&AbnormalKindCategory::Rhythm).unwrap()
    }

    pub fn all_effects_mut(&mut self) -> Vec<&mut Vec<AbnormalEffect>> {
        self.effects.values_mut().collect()
    }

    pub fn activate_effect_by_kind(&mut self, kind: &AbnormalKind) -> Option<AbnormalEffect> {
        let effects_vec = self.get_vec_by_kind_mut(&kind.category());

        let same_kind_effects: Vec<usize> = effects_vec
            .iter()
            .enumerate()
            .filter(|(_, effect)| effect.kind.is_same(kind))
            .map(|(idx, _)| idx)
            .collect();

        if same_kind_effects.is_empty() {
            return None;
        }

        // Find the effect with the highest magic level (or first if equal)
        if let Some(&best_effect_idx) = same_kind_effects.iter().min_by(|&&idx_a, &&idx_b| {
            let effect_a = &effects_vec[idx_a];
            let effect_b = &effects_vec[idx_b];

            // Compare by magic level (descending - higher is better)
            effect_b
                .skill()
                .magic_level()
                .cmp(&effect_a.skill().magic_level())
        }) {
            // First, deactivate all effects of the same kind
            for &idx in &same_kind_effects {
                effects_vec[idx].active = false;
            }

            // Then activate only the selected effect
            let was_active = effects_vec[best_effect_idx].active;
            effects_vec[best_effect_idx].active = true;

            if !was_active {
                Some(effects_vec[best_effect_idx])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Remove excess effects to maintain the maximum allowed count for each type
    fn remove_excess_effects(&mut self, variant: &AbnormalKindCategory) {
        let (current_count, max_count) = match variant {
            AbnormalKindCategory::Buff => (self.buffs_len(), self.max_buffs),
            AbnormalKindCategory::Debuff => (self.debuff_len(), self.max_debuffs),
            AbnormalKindCategory::Rhythm => (self.rhythm_len(), self.max_rhythm),
        };

        if current_count <= max_count {
            return;
        }

        let excess_count = current_count - max_count;
        let effects_vec = self.get_vec_by_kind_mut(variant);

        // Find active effects and remove the first ones (oldest)
        let mut removed_count = 0;
        effects_vec.retain(|effect| {
            if effect.active && removed_count < excess_count {
                log::debug!("Removing excess abnormal effect: {:?}", effect);
                removed_count += 1;
                false // Remove this effect
            } else {
                true // Keep this effect
            }
        });
    }

    fn get_vec_by_kind_mut(&mut self, variant: &AbnormalKindCategory) -> &mut Vec<AbnormalEffect> {
        self.effects.entry(*variant).or_default()
    }
}
