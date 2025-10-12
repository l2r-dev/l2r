use crate::{network::packets::server::StatusUpdate, object_id::ObjectId, stats::*};
use bevy::platform::collections::HashMap;
use bevy_common_assets::json::JsonAssetPlugin;
use derive_more::From;
use sea_orm::{
    TryGetError, TryGetable, Value,
    sea_query::{self, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::{Display, EnumIter, IntoEnumIterator};

mod cp;
mod hp;
mod mp;

pub use cp::*;
pub use hp::*;
pub use mp::*;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct VitalsChangeFlags(HashSet<VitalsStat>);

impl From<Vec<VitalsStat>> for VitalsChangeFlags {
    fn from(stats: Vec<VitalsStat>) -> Self {
        Self(HashSet::from_iter(stats))
    }
}

impl StatsChangeFlags<VitalsStat> for VitalsChangeFlags {
    fn set_changed(&mut self, stat: VitalsStat) {
        self.0.insert(stat);
    }

    fn has_changes(&self) -> bool {
        !self.0.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = &VitalsStat> + '_ {
        self.0.iter()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn consume(&mut self) -> Vec<VitalsStat> {
        let changes = self.iter().copied().collect();
        self.clear();
        changes
    }
}

pub struct VitalsStatsComponentsPlugin;
impl Plugin for VitalsStatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<LeveledVitalsStats>::new(&["json"]));

        app.add_event::<FullVitalsRestore>()
            .add_event::<Resurrect>();

        app.register_type::<VitalsStats>()
            .register_type::<Hp>()
            .register_type::<Mp>()
            .register_type::<Cp>();

        app.world_mut()
            .resource_mut::<StatFormulaRegistry>()
            .register_formula(VitalsStat::MaxHp.into(), Hp::formula)
            .register_formula(VitalsStat::MaxCp.into(), Cp::formula)
            .register_formula(VitalsStat::MaxMp.into(), Mp::formula);
    }
}

#[derive(Clone, Copy, Event)]
pub struct Resurrect;

#[derive(Clone, Copy, Debug, Event, From)]
pub struct FullVitalsRestore(Entity);

impl ContainsEntity for FullVitalsRestore {
    fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Clone, Component, Debug, Default, Deserialize, PartialEq, Reflect, Serialize)]
pub struct VitalsStats {
    #[serde(flatten)]
    current: FloatStats<VitalsStat>,
    #[serde(skip)]
    change_flags: VitalsChangeFlags,
}

impl VitalsStats {
    pub fn new(
        world: &World,
        class_id: ClassId,
        class_tree: &ClassTree,
        progress_level: &ProgressLevelStats,
        primal_stats: &PrimalStats,
        stat_modifiers: &StatModifiers,
    ) -> Self {
        let stats_table = world.resource::<StatsTable>();
        let stat_formula_registry = world.resource::<StatFormulaRegistry>();

        let base_stats = stats_table
            .vitals_stats_world(world, class_id, progress_level.level())
            .expect("Vitals stats asset must be loaded");

        let base_class = class_tree.get_base_class(class_id);

        let mut initial_stats = base_stats.clone();

        let params = StatsCalculateParams::new(
            stat_formula_registry,
            primal_stats,
            progress_level,
            base_class,
            None,
            stat_modifiers,
            true,
            false,
        );

        let changed = initial_stats
            .current
            .calculate(params, Some(base_stats.current()));

        Self {
            current: initial_stats.current.clone(),
            change_flags: VitalsChangeFlags::from(changed.unwrap_or_default()),
        }
    }

    pub fn calculate(
        &mut self,
        params: StatsCalculateParams,
        base_stats: Option<&FloatStats<VitalsStat>>,
    ) -> Option<Vec<VitalsStat>> {
        let changed_stats = self.current.calculate(params, base_stats);
        if let Some(ref stats) = changed_stats {
            for &stat in stats {
                self.change_flags.set_changed(stat);
            }
        }
        changed_stats
    }

    pub fn current(&self) -> &FloatStats<VitalsStat> {
        &self.current
    }

    /// Mark a stat as changed for network sync
    pub fn mark_stat_changed(&mut self, stat: VitalsStat) {
        self.change_flags.set_changed(stat);
    }

    /// Check if any stats have changed
    pub fn has_changes(&self) -> bool {
        self.change_flags.has_changes()
    }

    /// Get list of changed stats
    pub fn changed_stats(&self) -> impl Iterator<Item = &VitalsStat> {
        self.change_flags.iter()
    }

    pub fn changed_stats_vec(&self) -> Vec<VitalsStat> {
        self.changed_stats().copied().collect()
    }

    /// Consume and clear change tracking (for network sync)
    pub fn consume_changes(&mut self) -> Vec<VitalsStat> {
        self.change_flags.consume()
    }

    pub fn get(&self, stat: &VitalsStat) -> f32 {
        self.current.get(stat)
    }

    pub fn insert(&mut self, stat: VitalsStat, value: f32) {
        // Only mark as changed if the value actually changed
        let current_value = self.current.get(&stat);
        if current_value != value {
            self.current.insert(stat, value);
            self.change_flags.set_changed(stat);
        }
    }

    /// Clear all change tracking (call after network sync)
    pub fn clear_changes(&mut self) {
        self.change_flags.clear();
    }

    pub fn merge(&mut self, other: &VitalsStats) {
        for (stat, value) in other.current.iter() {
            self.insert(*stat, *value);
        }
    }

    pub fn percent_stat_damage(&mut self, stat: &VitalsStat, percent: f32) {
        let max = self.get(stat);
        let damage_amount = max * percent / 100.0;
        if damage_amount > 0.0 {
            self.stat_damage(stat, damage_amount);
        }
    }

    pub fn damage(&mut self, damage: f32, damage_cp: bool) {
        let rest = if damage_cp {
            self.stat_damage(&VitalsStat::Cp, damage)
        } else {
            damage
        };

        self.stat_damage(&VitalsStat::Hp, rest);
    }

    fn stat_damage(&mut self, stat: &VitalsStat, damage: f32) -> f32 {
        let current_value = self.get(stat);
        let rest = current_value - damage;
        if rest <= 0.0 {
            if current_value != 0.0 {
                self.insert(*stat, 0.0);
            }
            -rest
        } else {
            if current_value != rest {
                self.insert(*stat, rest);
            }
            0.0
        }
    }

    pub fn fill_current_from_max(&mut self) {
        let max_hp = self.get(&VitalsStat::MaxHp);
        let max_mp = self.get(&VitalsStat::MaxMp);
        let max_cp = self.get(&VitalsStat::MaxCp);

        let current_hp = self.get(&VitalsStat::Hp);
        let current_mp = self.get(&VitalsStat::Mp);
        let current_cp = self.get(&VitalsStat::Cp);

        if current_hp != max_hp {
            self.insert(VitalsStat::Hp, max_hp);
        }
        if current_mp != max_mp {
            self.insert(VitalsStat::Mp, max_mp);
        }
        if current_cp != max_cp {
            self.insert(VitalsStat::Cp, max_cp);
        }
    }

    pub fn kill(&mut self) {
        let current_hp = self.get(&VitalsStat::Hp);
        if current_hp != 0.0 {
            self.insert(VitalsStat::Hp, 0.0);
        }
    }

    pub fn dead(&self) -> bool {
        self.get(&VitalsStat::Hp) <= 0.0
    }

    pub fn diff_status_update(&mut self, object_id: ObjectId) -> Option<StatusUpdate> {
        if !self.has_changes() {
            return None;
        }
        let mut status_update = StatusUpdate::new(object_id);
        for variant in self.changed_stats() {
            match variant {
                VitalsStat::Hp
                | VitalsStat::Mp
                | VitalsStat::Cp
                | VitalsStat::MaxHp
                | VitalsStat::MaxMp
                | VitalsStat::MaxCp => {
                    status_update.add(variant.into(), self.get(variant) as u32);
                }
                _ => {}
            }
        }
        self.clear_changes();
        Some(status_update)
    }

    pub fn current_max(&self, stat: &VitalsStat) -> Option<(f32, f32)> {
        let current_stat = stat.has_current()?;
        Some((self.get(&current_stat), self.get(stat)))
    }

    pub fn cp_hp_mp_current_max(&self) -> Option<((f32, f32), (f32, f32), (f32, f32))> {
        Some((
            self.current_max(&VitalsStat::MaxCp)?,
            self.current_max(&VitalsStat::MaxHp)?,
            self.current_max(&VitalsStat::MaxMp)?,
        ))
    }

    pub fn apply_operation(&mut self, stat: &VitalsStat, operation: &StatsOperation<f32>) {
        self.current.apply_operation(stat, operation);
        self.change_flags.set_changed(*stat);
    }

    pub fn test_data() -> VitalsStats {
        let mut stats = VitalsStats::default();
        stats.current.insert(VitalsStat::MaxHp, 2000.0);
        stats.current.insert(VitalsStat::MaxMp, 2000.0);
        stats.current.insert(VitalsStat::MaxCp, 2000.0);
        stats.current.insert(VitalsStat::HpRegen, 2.0);
        stats.current.insert(VitalsStat::MpRegen, 0.9);
        stats.current.insert(VitalsStat::CpRegen, 2.0);
        // Note: test_data doesn't set change flags since it's creating initial test state
        stats
    }
}

impl From<VitalsStats> for Value {
    fn from(stats: VitalsStats) -> Self {
        let mut db_stats = HashMap::new();

        db_stats.insert(VitalsStat::Cp, stats.get(&VitalsStat::Cp));
        db_stats.insert(VitalsStat::Hp, stats.get(&VitalsStat::Hp));
        db_stats.insert(VitalsStat::Mp, stats.get(&VitalsStat::Mp));

        Value::Json(Some(Box::new(serde_json::to_value(db_stats).unwrap())))
    }
}

impl TryGetable for VitalsStats {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: serde_json::Value = res.try_get_by(idx)?;
        serde_json::from_value(value).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(
                "Failed to deserialize VitalsStats from JSON".to_owned(),
            ))
        })
    }
}

impl ValueType for VitalsStats {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::Json(Some(json)) = v {
            serde_json::from_value(*json).map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(VitalsStats).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::Json
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::Json
    }
}

#[derive(Asset, Clone, Debug, Default, Deref, Deserialize, TypePath)]
pub struct LeveledVitalsStats(HashMap<Level, VitalsStats>);

#[derive(Default, Deref, DerefMut, From, Resource)]
pub struct VitalsStatsHandlers(HashMap<ClassId, Handle<LeveledVitalsStats>>);

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize, Display,
)]
pub enum VitalsStat {
    Hp,
    Mp,
    Cp,
    MaxHp,
    MaxMp,
    MaxCp,
    HpRegen,
    MpRegen,
    CpRegen,
    MaxRecoverableHp,
    MaxRecoverableMp,
    MaxRecoverableCp,
    ManaCharge,
    HealEffect,
}

impl VitalsStat {
    pub fn has_current(&self) -> Option<VitalsStat> {
        match self {
            VitalsStat::MaxHp => Some(VitalsStat::Hp),
            VitalsStat::MaxMp => Some(VitalsStat::Mp),
            VitalsStat::MaxCp => Some(VitalsStat::Cp),
            _ => None,
        }
    }
}

impl StatTrait for VitalsStat {
    fn calculate_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|&stat| {
            matches!(
                stat,
                VitalsStat::MaxHp
                    | VitalsStat::MaxMp
                    | VitalsStat::MaxCp
                    | VitalsStat::HpRegen
                    | VitalsStat::MpRegen
                    | VitalsStat::CpRegen
            )
        })
    }

    fn has_max_pair(&self) -> Option<Self> {
        match self {
            VitalsStat::Hp => Some(VitalsStat::MaxHp),
            VitalsStat::Mp => Some(VitalsStat::MaxMp),
            VitalsStat::Cp => Some(VitalsStat::MaxCp),
            _ => None,
        }
    }
}
