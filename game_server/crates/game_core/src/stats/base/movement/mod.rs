use crate::stats::*;
use bevy::platform::collections::HashMap;
use l2r_core::model::base_class::BaseClass;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

mod movable;

pub use movable::*;

pub struct MovementStatsComponentsPlugin;
impl Plugin for MovementStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MovementStats>()
            .register_type::<MovementStat>()
            .register_type::<Movable>();

        let mut formula_registry = app.world_mut().resource_mut::<StatFormulaRegistry>();
        for stat in MovementStat::iter() {
            formula_registry.register_formula(stat.into(), MovementStats::formula);
        }
    }
}

#[derive(
    EnumIter,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Reflect,
    IntoPrimitive,
)]
#[repr(u32)]
pub enum MovementStat {
    Walk,
    #[default]
    Run,
    Swim,
    FastSwim,
    Fly,
    FastFly,
    Fall,
}
impl StatTrait for MovementStat {
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::from(45).unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Default, Display, Eq, Hash, IntoPrimitive, PartialEq, Reflect)]
#[repr(u8)]
pub enum MoveState {
    #[default]
    Ground,
    Water,
    Air,
}
impl From<MovementStat> for MoveState {
    fn from(move_type: MovementStat) -> Self {
        match move_type {
            MovementStat::Walk | MovementStat::Run => MoveState::Ground,
            MovementStat::Swim | MovementStat::FastSwim => MoveState::Water,
            MovementStat::Fly | MovementStat::FastFly => MoveState::Air,
            MovementStat::Fall => MoveState::Air,
        }
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut, PartialEq, Reflect, Serialize)]
pub struct MovementStats(UIntStats<MovementStat>);

impl AsRef<GenericStats<MovementStat, u32>> for MovementStats {
    fn as_ref(&self) -> &GenericStats<MovementStat, u32> {
        &self.0
    }
}

impl MovementStats {
    fn formula(args: FormulaArguments) -> f32 {
        let dex_bouns = args.primal.typed::<DEX>(&PrimalStat::DEX).bonus();
        args.base_value * dex_bouns
    }
}

impl<'de> Deserialize<'de> for MovementStats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let partial: HashMap<MovementStat, u32> = HashMap::deserialize(deserializer)?;

        let mut stats = MovementStats::default();

        // take Walk and Run cause they are filled in files, and set others from them
        let walk = partial
            .get(&MovementStat::Walk)
            .copied()
            .unwrap_or_default();

        let run = partial.get(&MovementStat::Run).copied().unwrap_or_default();
        for stat in MovementStat::iter() {
            let value = match stat {
                MovementStat::Run => run,
                _ => walk,
            };
            stats.insert(stat, value);
        }

        Ok(stats)
    }
}
