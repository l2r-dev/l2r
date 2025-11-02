use super::{model::CollisionSize, monster_ai::MonsterAiParams};
use crate::{
    abnormal_effects::AbnormalEffects,
    action::target::Targetable,
    attack::AttackingList,
    collision_layers::Layer,
    items::PaperDoll,
    object_id::{ObjectId, ObjectIdManager},
    stats::*,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use l2r_core::model::{base_class::BaseClass, race::Race};

#[derive(Bundle, Debug)]
pub struct Bundle {
    pub id: super::Id,
    pub object_id: ObjectId,
    pub kind: super::Kind,
    pub primal_stats: PrimalStats,
    pub attack_stats: AttackStats,
    pub critical_stats: CriticalStats,
    pub defence_stats: DefenceStats,
    pub vitals_stats: VitalsStats,
    pub progress_level: ProgressLevelStats,
    pub stat_modifiers: StatModifiers,
    pub name: Name,
    pub title: NameTitle,
    pub race: Race,
    pub gender: Gender,
    pub reward: ProgressReward,
    pub movable: Movable,
    // pub skill_list: Vec<Skill>,
    pub ai: MonsterAiParams,
    pub transform: Transform,
    pub collision: CollisionSize,
    pub collider: Collider,
    pub collision_layers: CollisionLayers,
    pub pvp_stats: PvpStats,
    pub visibility: EncountersVisibility,
    pub attackers_list: AttackingList,
    pub class_id: ClassId,
    pub base_class: BaseClass,
    pub attack_effects: AttackEffects,
    pub defence_effects: DefenceEffects,
    pub abnormal_effects: AbnormalEffects,
    pub other_stats: OtherStats,
    pub paper_doll: PaperDoll,
    pub targetable: Targetable,
}

impl Bundle {
    pub fn new(
        id: super::Id,
        npc: super::Model,
        transform: Transform,
        formula_registry: &StatFormulaRegistry,
        object_id_manager: &mut ObjectIdManager,
    ) -> Self {
        let progress_level = ProgressLevelStats::new(npc.level);
        let stat_modifiers = StatModifiers::default();
        let default_primal_stats = PrimalStats::default();
        let mut primal_stats = PrimalStats::default();
        let default_base_class = BaseClass::default();

        let params = StatsCalculateParams::new(
            formula_registry,
            &default_primal_stats,
            &progress_level,
            default_base_class,
            None,
            &stat_modifiers,
            false,
            false,
        );
        primal_stats.calculate(params, None);

        let mut vitals_stats = npc.stats.vitals.clone();
        let params = StatsCalculateParams::new(
            formula_registry,
            &primal_stats,
            &progress_level,
            default_base_class,
            None,
            &stat_modifiers,
            false,
            false,
        );
        vitals_stats.calculate(params, Some(npc.stats.vitals.current()));
        vitals_stats.fill_current_from_max();

        let movable = Movable::new(npc.stats.speed);
        let collider_info = npc.collision.get(ColliderSize::Normal);
        let collider = Collider::capsule(collider_info.radius, collider_info.height);
        let object_id = object_id_manager.next_id();
        Self {
            id,
            object_id,
            kind: npc.kind,
            primal_stats,
            attack_stats: npc.stats.attack,
            critical_stats: npc.stats.critical,
            defence_stats: npc.stats.defence,
            vitals_stats,
            progress_level,
            stat_modifiers,
            name: Name::new(npc.name.unwrap_or_default()),
            title: NameTitle::new(npc.title.unwrap_or_default()),
            race: npc.race,
            gender: npc.gender,
            collision: npc.collision,
            collider,
            collision_layers: Layer::character(),
            reward: npc.reward.unwrap_or_default(),
            movable,
            ai: npc.ai.unwrap_or_default(),
            transform,
            pvp_stats: PvpStats::default(),
            visibility: EncountersVisibility::default(),
            attackers_list: AttackingList::default(),
            class_id: ClassId::default(),
            base_class: BaseClass::default(),
            attack_effects: AttackEffects::default(),
            defence_effects: DefenceEffects::default(),
            abnormal_effects: AbnormalEffects::default(),
            other_stats: OtherStats::default(),
            paper_doll: PaperDoll::default(),
            targetable: Targetable,
        }
    }
}
