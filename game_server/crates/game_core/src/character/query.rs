use crate::{
    action::wait_kind::Sit,
    attack::{Dead, InCombat},
    character::model::ModelUpdate,
    items::PaperDoll,
    object_id::ObjectId,
    stats::*,
};
use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use l2r_core::model::{base_class::BaseClass, race::Race};
use spatial::GameVec3;

#[derive(QueryData)]
#[query_data(derive(Clone))]
pub struct Query<'a> {
    pub name: &'a Name,
    pub title: &'a NameTitle,
    pub entity: Entity,
    pub object_id: &'a ObjectId,
    pub movable: &'a Movable,
    pub transform: &'a Transform,
    pub race: &'a Race,
    pub sub_class: &'a SubClass,
    pub appearance: &'a super::appearance::Appearance,
    pub base_class: &'a BaseClass,
    pub pvp_stats: &'a PvpStats,
    pub progress_stats: &'a ProgressStats,
    pub progress_level: &'a ProgressLevelStats,
    pub primal_stats: &'a PrimalStats,
    pub attack_stats: &'a AttackStats,
    pub defence_stats: &'a DefenceStats,
    pub critical_stats: &'a CriticalStats,
    pub inventory_stats: &'a InventoryStats,
    pub vitals_stats: &'a VitalsStats,
    pub paperdoll: &'a PaperDoll,
    pub collider: &'a Collider,
    pub visibility: &'a EncountersVisibility,
    pub dead: Option<&'a Dead>,
    pub in_combat: Option<&'a InCombat>,
    pub sitting: Option<&'a Sit>,
}

impl<'a, 'b> From<&'a QueryItem<'a, 'b>> for ModelUpdate {
    fn from(character: &'a QueryItem<'a, 'b>) -> Self {
        Self {
            title: character.title.to_string().clone(),
            position: GameVec3::from(character.transform.translation),
            exp: character.progress_stats.exp() as i64,
            sp: character.progress_stats.sp() as i32,
            vitals: character.vitals_stats.clone(),
            is_last_active: true,
        }
    }
}
