use bevy::prelude::*;
use game_core::stats::{
    AttackEffects, DefenceEffects, EffectKind, EffectKindComponentsPlugin, Weakness,
};
use strum::IntoEnumIterator;

pub struct EffectKindPlugin;
impl Plugin for EffectKindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectKindComponentsPlugin);
    }
}

pub fn get_attack_trait_bonus(attacker_entity: EntityRef, target_entity: EntityRef) -> f32 {
    let Some(attacker_attack_effects) = attacker_entity.get::<AttackEffects>() else {
        return 0.0;
    };

    let Some(target_defence_effects) = target_entity.get::<DefenceEffects>() else {
        return 0.0;
    };

    calc_attack_trait_bonus(attacker_attack_effects, target_defence_effects)
}

fn calc_attack_trait_bonus(
    attacker_attack_effects: &AttackEffects,
    target_defence_effects: &DefenceEffects,
) -> f32 {
    let weapon_trait_bonus =
        calc_weapon_trait_bonus(attacker_attack_effects, target_defence_effects);
    if weapon_trait_bonus == 0.0 {
        return 0.0;
    }

    let mut weakness_bonus = 1.0;
    for weakness in Weakness::iter() {
        let effect_kind = EffectKind::Weakness(weakness);
        weakness_bonus *= calc_general_trait_bonus(
            attacker_attack_effects,
            target_defence_effects,
            effect_kind,
            true,
        );
        if weakness_bonus == 0.0 {
            return 0.0;
        }
    }

    let result = weapon_trait_bonus * weakness_bonus;
    result.clamp(0.05, 2.0)
}

fn calc_general_trait_bonus(
    attacker_attack_effects: &AttackEffects,
    target_defence_effects: &DefenceEffects,
    effect_kind: EffectKind,
    ignore_resistance: bool,
) -> f32 {
    if target_defence_effects.is_invulnerable(effect_kind) {
        return 0.0;
    }

    match effect_kind {
        EffectKind::Weakness(_) => {
            if !attacker_attack_effects.has(effect_kind) || !target_defence_effects.has(effect_kind)
            {
                return 1.0;
            }
        }
        EffectKind::Resistance(_) => {
            if ignore_resistance {
                return 1.0;
            }
        }
        _ => {}
    }

    let result =
        (attacker_attack_effects.get(effect_kind) - target_defence_effects.get(effect_kind)) + 1.0;
    result.clamp(0.05, 2.0)
}

#[inline]
fn calc_weapon_trait_bonus(
    attacker_attack_effects: &AttackEffects,
    target_defence_effects: &DefenceEffects,
) -> f32 {
    2.0 - target_defence_effects.get(EffectKind::Weapon(attacker_attack_effects.get_weapon()))
}
