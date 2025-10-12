use super::summon::Summon;
use crate::{
    attack::{Attackable, Dead, InCombat},
    items::PaperDoll,
    npc::id::DisplayId,
    object_id::ObjectId,
    stats::*,
};
use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

#[derive(QueryData)]
#[query_data(derive(Debug, Clone))]
pub struct NpcQuery<'a> {
    pub entity: Entity,
    pub object_id: &'a ObjectId,
    pub id: &'a super::Id,
    pub display_id: Option<&'a DisplayId>,
    pub kind: &'a super::Kind,
    pub attackable: Option<&'a Attackable>,
    pub in_combat: Option<&'a InCombat>,
    pub dead: Option<&'a Dead>,
    pub summon: Option<&'a Summon>,
    pub transform: &'a Transform,
    pub attack_stats: &'a AttackStats,
    pub critical_stats: &'a CriticalStats,
    pub defence_stats: &'a DefenceStats,
    pub condition: &'a VitalsStats,
    pub movable: &'a Movable,
    pub collider: &'a Collider,
    pub name: &'a Name,
    pub title: Option<&'a NameTitle>,
    pub progress_reward: &'a ProgressReward,
    pub pvp_stats: &'a PvpStats,
    pub paperdoll_items: Option<&'a PaperDoll>,
    pub visibility: &'a Visibility,
}
