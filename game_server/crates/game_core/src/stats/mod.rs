use crate::{
    character::Character,
    encounters::EnteredWorld,
    items::PaperDoll,
    network::packets::server::UserInfoUpdated,
    npc::{self, kind::Pet},
};
use bevy::prelude::*;
use bevy_ecs::{query::QueryData, system::SystemParam};
use l2r_core::model::{base_class::BaseClass, race::Race};
use num_enum::TryFromPrimitive;
use num_traits::{Num, NumCast};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};
use strum::{EnumCount, IntoEnumIterator};

mod attack;
mod attribute;
mod base;
mod calc;
mod collider;
mod critical;
mod defence;
mod element;
mod gender;
mod inventory;
mod kind;
mod other;
mod progress;
mod pvp;
mod race;
mod sub_class;
mod table;
mod title;
mod visible;

pub use attack::*;
pub use attribute::*;
pub use base::*;
pub use calc::*;
pub use collider::*;
pub use critical::*;
pub use defence::*;
pub use element::*;
pub use gender::*;
pub use inventory::*;
pub use kind::*;
pub use other::*;
pub use progress::*;
pub use pvp::*;
pub use race::*;
pub use sub_class::*;
pub use table::*;
pub use title::*;
pub use visible::*;

pub struct StatsComponentsPlugin;
impl Plugin for StatsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StatsTable::default());

        app.add_plugins(InventoryStatsComponentsPlugin);

        app.add_event::<UserInfoUpdated>();

        app.register_type::<NameTitle>()
            .register_type::<SubClass>()
            .register_type::<ItemElementsInfo>();

        l2r_core::register_optional_types!(app, ItemElementsInfo);
    }
}

pub trait StatValue: Num + Copy + NumCast + PartialOrd + Default {}
impl StatValue for f32 {}
impl StatValue for u32 {}
impl StatValue for f64 {}

pub trait StatTrait:
    Into<StatKind>
    + IntoEnumIterator
    + Eq
    + Hash
    + Copy
    + Serialize
    + DeserializeOwned
    + EnumCount
    + Into<usize>
    + TryFromPrimitive
    + 'static
{
    fn default_value<V: StatValue>(&self, _base_class: BaseClass) -> V {
        V::default()
    }
    fn max_value<V: StatValue>(&self, _base_class: BaseClass) -> V;
    fn with_doll<V: StatValue>(&self, base_class: BaseClass, _paper_doll: &PaperDoll) -> V {
        self.default_value(base_class)
    }
    fn calculate_iter() -> impl Iterator<Item = Self> {
        Self::iter()
    }
    fn has_max_pair(&self) -> Option<Self> {
        None
    }
}

pub trait Stats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    fn get(&self, stat: S) -> V;

    fn typed<T>(&self, stat: S) -> T
    where
        T: Default + From<V>;

    fn merge(&mut self, other: &Self);

    fn insert(&mut self, stat: S, value: V);

    fn changed(&self, other: &Self) -> bool;

    fn changed_variants(&self, other: &Self) -> Vec<S>;

    fn iter(&self) -> impl Iterator<Item = (S, V)>;

    fn apply_operation(&mut self, stat: S, operation: &StatsOperation<V>) {
        let current = self.get(stat);
        let new_value = operation.apply(current);
        if new_value != current {
            self.insert(stat, new_value);
        }
    }

    fn apply_operations(&mut self, operations: &[(S, StatsOperation<V>)]) {
        for (stat, operation) in operations {
            self.apply_operation(*stat, operation);
        }
    }

    fn calculate(
        &mut self,
        params: StatsCalculateParams,
        base_stats: Option<&Self>,
    ) -> Option<Vec<S>>;

    fn calculate_fallback_stat_value(&self, stat: S, params: &StatsCalculateParams) -> V;
}

pub type CalcChangedCoponents = Or<(
    Changed<Race>,
    Changed<BaseClass>,
    Changed<SubClass>,
    Changed<StatModifiers>,
    Changed<ProgressLevelStats>,
    Changed<PrimalStats>,
)>;

#[derive(SystemParam)]
pub struct StatsCalcParams<'w, 's, T: Component<Mutability = bevy_ecs::component::Mutable>> {
    pub stats_table: StatsTableQuery<'w>,
    pub calc_components_changed: Query<'w, 's, Entity, CalcChangedCoponents>,
    pub user_info_updated: EventWriter<'w, UserInfoUpdated>,
    pub formula_registry: Res<'w, StatFormulaRegistry>,
    pub npc_info: npc::RegionalNpcInfoQuery<'w, 's>,
    pub query: Query<
        'w,
        's,
        (
            StatsCalcQuery<'static>,
            Mut<'static, T>,
            Option<Ref<'static, EnteredWorld>>,
        ),
    >,
}

#[derive(QueryData)]
pub struct StatsCalcQuery<'a> {
    pub primal_stats: Ref<'a, PrimalStats>,
    pub progress_level: Ref<'a, ProgressLevelStats>,
    pub base_class: Ref<'a, BaseClass>,
    pub stat_modifiers: Ref<'a, StatModifiers>,
    pub paper_doll: Ref<'a, PaperDoll>,
    pub pet: Has<Pet>,
    pub character: Has<Character>,
    pub sub_class: Option<Ref<'a, SubClass>>,
    pub race: Ref<'a, Race>,
}

#[derive(QueryData)]
pub struct StatsCalcNoPrimalQuery<'a> {
    pub progress_level: Ref<'a, ProgressLevelStats>,
    pub base_class: Ref<'a, BaseClass>,
    pub stat_modifiers: Ref<'a, StatModifiers>,
    pub paper_doll: Ref<'a, PaperDoll>,
    pub pet: Has<Pet>,
    pub character: Has<Character>,
}

pub struct StatsCalculateParams<'a> {
    formula_registry: &'a StatFormulaRegistry,
    primal_stats: &'a PrimalStats,
    progress_level: &'a ProgressLevelStats,
    base_class: BaseClass,
    paper_doll: Option<&'a PaperDoll>,
    stat_modifiers: &'a StatModifiers,
    is_character: bool,
    is_pet: bool,
}

impl<'a> StatsCalculateParams<'a> {
    pub fn new(
        formula_registry: &'a StatFormulaRegistry,
        primal_stats: &'a PrimalStats,
        progress_level: &'a ProgressLevelStats,
        base_class: BaseClass,
        paper_doll: Option<&'a PaperDoll>,
        stat_modifiers: &'a StatModifiers,
        is_character: bool,
        is_pet: bool,
    ) -> Self {
        Self {
            formula_registry,
            primal_stats,
            progress_level,
            base_class,
            paper_doll,
            stat_modifiers,
            is_character,
            is_pet,
        }
    }

    pub fn from_query(
        query: &'a StatsCalcQueryItem,
        formula_registry: &'a StatFormulaRegistry,
    ) -> Self {
        Self {
            formula_registry,
            primal_stats: &query.primal_stats,
            progress_level: &query.progress_level,
            base_class: *query.base_class,
            paper_doll: Some(&query.paper_doll),
            stat_modifiers: &query.stat_modifiers,
            is_character: query.character,
            is_pet: query.pet,
        }
    }

    pub fn from_query_no_primal(
        query: &'a StatsCalcNoPrimalQueryItem,
        primal_stats: &'a PrimalStats,
        formula_registry: &'a StatFormulaRegistry,
    ) -> Self {
        Self {
            formula_registry,
            primal_stats,
            progress_level: &query.progress_level,
            base_class: *query.base_class,
            paper_doll: Some(&query.paper_doll),
            stat_modifiers: &query.stat_modifiers,
            is_character: query.character,
            is_pet: query.pet,
        }
    }

    pub fn formula_registry(&self) -> &StatFormulaRegistry {
        self.formula_registry
    }

    pub fn primal_stats(&self) -> &PrimalStats {
        self.primal_stats
    }

    pub fn progress_level(&self) -> &ProgressLevelStats {
        self.progress_level
    }

    pub fn base_class(&self) -> BaseClass {
        self.base_class
    }

    pub fn paper_doll(&self) -> Option<&'a PaperDoll> {
        self.paper_doll
    }

    pub fn stat_modifiers(&self) -> &StatModifiers {
        self.stat_modifiers
    }

    pub fn is_character(&self) -> bool {
        self.is_character
    }

    pub fn is_pet(&self) -> bool {
        self.is_pet
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    values: Vec<V>,
    #[reflect(ignore)]
    _phantom: std::marker::PhantomData<S>,
}

impl<S, V> Default for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    fn default() -> Self {
        let base_class = BaseClass::default();
        let values = S::iter()
            .map(|stat| stat.default_value(base_class))
            .collect();
        Self {
            values,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, V> Index<S> for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    type Output = V;

    fn index(&self, stat: S) -> &Self::Output {
        let index: usize = stat.into();
        debug_assert!(
            index < self.values.len(),
            "Stat index {} out of bounds (len: {})",
            index,
            self.values.len()
        );
        &self.values[index]
    }
}

impl<S, V> IndexMut<S> for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    fn index_mut(&mut self, stat: S) -> &mut Self::Output {
        let index: usize = stat.into();
        debug_assert!(
            index < self.values.len(),
            "Stat index {} out of bounds (len: {})",
            index,
            self.values.len()
        );
        &mut self.values[index]
    }
}

impl<S, V> Stats<S, V> for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    fn get(&self, stat: S) -> V {
        let index: usize = stat.into();
        self.values.get(index).copied().unwrap_or_default()
    }

    fn typed<T>(&self, stat: S) -> T
    where
        T: Default + From<V>,
    {
        T::from(self.get(stat))
    }

    fn merge(&mut self, other: &Self) {
        for (self_val, &other_val) in self.values.iter_mut().zip(other.values.iter()) {
            if other_val != V::zero() {
                *self_val = other_val;
            }
        }
    }

    fn insert(&mut self, stat: S, value: V) {
        let index: usize = stat.into();
        self.values[index] = value;
    }

    fn changed(&self, other: &Self) -> bool {
        self.values != other.values
    }

    fn changed_variants(&self, other: &Self) -> Vec<S> {
        let mut changed = Vec::with_capacity(S::COUNT);
        for stat in S::iter() {
            if self[stat] != other[stat] {
                changed.push(stat);
            }
        }
        changed
    }

    fn iter(&self) -> impl Iterator<Item = (S, V)> {
        S::iter().map(move |stat| (stat, self.get(stat)))
    }

    fn calculate_fallback_stat_value(&self, stat: S, params: &StatsCalculateParams) -> V {
        // First, check if there's a top-level set modifier for this stat
        // For example, equipped weapon sets attack power directly
        if let Some(modifier_value) = params.stat_modifiers().get_top_set_modifier(stat.into()) {
            return V::from(modifier_value).unwrap_or_default();
        }

        // If no modifier, determine value from paper doll or use default
        if let Some(paper_doll) = params.paper_doll() {
            // Calculate from equipped items, needed for armor, to get correct defense values
            // Because we need to 'unequip' base class default armor when calculating defense
            stat.with_doll(params.base_class(), paper_doll)
        } else {
            stat.default_value(params.base_class())
        }
    }

    fn calculate(
        &mut self,
        params: StatsCalculateParams,
        base_stats: Option<&Self>,
    ) -> Option<Vec<S>> {
        let mut changed_stats = Vec::with_capacity(S::COUNT / 2 + 1);
        for stat in S::calculate_iter() {
            // Determine initial value from base stats or calculate fallback value
            let init_value = if let Some(base_stats) = base_stats {
                base_stats.get(stat)
            } else {
                self.calculate_fallback_stat_value(stat, &params)
            };
            let init_value = NumCast::from(init_value).unwrap_or(0.0f32);
            let computed_value = params
                .formula_registry()
                .calculate_final_value(stat, FormulaArguments::from_params(init_value, &params));
            let mut final_value = V::from(computed_value).unwrap_or_default();

            // Apply stat capping for current/max stat pairs (e.g., current HP vs max HP)
            if let Some(max_stat) = stat.has_max_pair() {
                let max_value = self.get(max_stat);
                if final_value > max_value {
                    final_value = max_value;
                }
            }

            // Ensure stat does not exceed its defined maximum value
            let stat_max_value = stat.max_value(params.base_class());
            if final_value > stat_max_value {
                final_value = stat_max_value;
            }

            // Update stat if value changed and track changes for notification purposes
            if self.get(stat) != final_value {
                changed_stats.push(stat);
                self.insert(stat, final_value);
            }
        }
        if changed_stats.is_empty() {
            None
        } else {
            Some(changed_stats)
        }
    }
}

impl<S, V> Serialize for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue + Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(S::COUNT))?;
        for stat in S::iter() {
            let value = self[stat];
            if value != V::default() {
                map.serialize_entry(&stat, &value)?;
            }
        }
        map.end()
    }
}

impl<'de, S, V> Deserialize<'de> for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct GenericStatsVisitor<S, V> {
            _phantom: std::marker::PhantomData<(S, V)>,
        }

        impl<'de, S, V> Visitor<'de> for GenericStatsVisitor<S, V>
        where
            S: StatTrait,
            V: StatValue + Deserialize<'de>,
        {
            type Value = GenericStats<S, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of stat variants to values")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut stats = GenericStats::default();
                while let Some((key, value)) = map.next_entry::<S, V>()? {
                    stats[key] = value;
                }
                Ok(stats)
            }
        }

        deserializer.deserialize_map(GenericStatsVisitor {
            _phantom: std::marker::PhantomData,
        })
    }
}

pub type FloatStats<S> = GenericStats<S, f32>;
pub type DoubleStats<S> = GenericStats<S, f64>;
pub type IntStats<S> = GenericStats<S, i32>;
pub type UIntStats<S> = GenericStats<S, u32>;

impl<S, V> AsRef<GenericStats<S, V>> for GenericStats<S, V>
where
    S: StatTrait,
    V: StatValue,
{
    fn as_ref(&self) -> &GenericStats<S, V> {
        self
    }
}

pub trait StatsChangeFlags<S>
where
    S: StatTrait + 'static,
{
    fn clear(&mut self);
    fn has_changes(&self) -> bool;
    fn set_changed(&mut self, stat: S);
    fn iter(&self) -> impl Iterator<Item = &S> + '_;
    fn consume(&mut self) -> Vec<S>;
}
