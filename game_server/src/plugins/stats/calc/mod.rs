use bevy::prelude::*;
use game_core::stats::{StatFormulaRegistry, StatsOperation};

mod calc_crit;
mod effect_kind;
mod hit_miss;
mod modifiers;
mod p_atk_damage;
mod shield_result;

pub use calc_crit::*;
pub use effect_kind::*;
pub use hit_miss::*;
use modifiers::*;
pub use p_atk_damage::*;
pub use shield_result::*;

pub struct StatCalculationPlugin;

impl Plugin for StatCalculationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StatsOperation<u32>>()
            .register_type::<StatsOperation<f32>>();

        app.init_resource::<StatFormulaRegistry>();

        app.add_plugins(StatModifiersPlugin)
            .add_plugins(ShieldResultPlugin)
            .add_plugins(EffectKindPlugin);
    }
}
