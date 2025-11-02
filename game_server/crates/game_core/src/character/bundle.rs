use crate::{
    abnormal_effects::AbnormalEffects,
    action::{target::Targetable, wait_kind::WaitKind},
    character::{self, model::Model},
    collision_layers::Layer,
    encounters::KnownEntities,
    items::{self, Inventory, PaperDoll},
    object_id::ObjectId,
    skills::SkillList,
    stats::{NameTitle, *},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use l2r_core::model::{base_class::BaseClass, race::Race, session::SessionId};
use spatial::GameVec3;

#[derive(Bundle, Clone, Debug, Reflect)]
pub struct Bundle {
    pub id: ObjectId,
    pub character: super::Character,
    pub name: Name,
    pub title: NameTitle,
    pub session_id: SessionId,
    pub movable: Movable,
    #[reflect(ignore)]
    pub collider: Collider,
    pub collision_layers: CollisionLayers,
    pub primal_stats: PrimalStats,
    pub base_class: BaseClass,
    pub attack_stats: AttackStats,
    pub defence_stats: DefenceStats,
    pub critical_stats: CriticalStats,
    pub inventory_stats: InventoryStats,
    pub vitals_stats: VitalsStats,
    pub progress_stats: ProgressStats,
    pub progress_level: ProgressLevelStats,
    pub other_stats: OtherStats,
    pub stat_modifiers: StatModifiers,
    pub pvp: PvpStats,
    pub appearance: super::Appearance,
    pub race: Race,
    pub sub_class: SubClass,
    pub transform: Transform,
    pub last_known_pos: LastKnownPosition,
    pub paper_doll: PaperDoll,
    pub delete_timer: super::DeleteTimer,
    pub visibility: EncountersVisibility,
    pub skill_list: SkillList,
    pub known_entities: KnownEntities,
    pub inventory: Inventory,
    pub wait_kind: WaitKind,
    pub attack_effects: AttackEffects,
    pub defence_effects: DefenceEffects,
    pub abnormal_effects: AbnormalEffects,
    pub targetable: Targetable,
}

#[allow(clippy::too_many_arguments)]
impl Bundle {
    pub fn new(
        db_model: Model,
        paper_doll: PaperDoll,
        session_id: SessionId,
        world: &World,
    ) -> Self {
        let stats_table = world.resource::<StatsTable>();
        let class_tree = stats_table.class_tree_world(world);
        let race_stats = stats_table.race_stats_world(world);
        let stat_formula_registry = world.resource::<StatFormulaRegistry>();

        let base_class = class_tree.get_base_class(db_model.class_id);
        let progress_stats = ProgressStats::new(db_model.exp as u64, db_model.sp as u32);
        let progress_level = ProgressLevelStats::new(progress_stats.calculate_level_by_exp());
        let base_class_stats = race_stats.get(db_model.race, base_class);
        let mut primal_stats = base_class_stats.primal_stats.clone();
        let default_primal_stats = base_class_stats.primal_stats.clone();
        let movable = Movable::from(base_class_stats);
        let skill_list = SkillList::default();
        let position = GameVec3::new(db_model.x, db_model.y, db_model.z);

        let mut other_stats = OtherStats::default();
        other_stats.insert(OtherStat::Breath, base_class_stats.breath as f32);
        other_stats.insert(OtherStat::BreathMax, base_class_stats.breath as f32);

        let mut stat_modifiers = StatModifiers::default();

        let items_data_table = world.resource::<items::ItemsDataTable>();
        let items_data_assets = world.resource::<Assets<items::ItemsInfo>>();

        // Get all paperdoll items and apply their stats
        for slot_item in paper_doll.iter() {
            let Some(unique_item) = slot_item.unique_item() else {
                continue;
            };
            let item = unique_item.item();

            let Ok(item_info) = items_data_table.get_item_info(item.id(), items_data_assets) else {
                continue;
            };

            if let Some(stats) = item_info.stats_modifiers() {
                stat_modifiers.merge(&stats);
            }
        }

        let params = StatsCalculateParams::new(
            stat_formula_registry,
            &default_primal_stats,
            &progress_level,
            base_class,
            None,
            &stat_modifiers,
            true,
            false,
        );

        primal_stats.calculate(params, Some(&default_primal_stats));

        let sub_class = SubClass::from((db_model.sub_class, db_model.class_id));

        let mut vitals_stats = VitalsStats::new(
            world,
            db_model.class_id,
            class_tree,
            &progress_level,
            &primal_stats,
            &stat_modifiers,
        );

        // Per-level stats stored in assets, but current stats are stored in db, so we need to merge them
        vitals_stats.merge(&db_model.vitals);

        Self {
            id: db_model.id,
            character: super::Character,
            name: Name::new(db_model.name),
            title: NameTitle::new(db_model.title),
            session_id,
            movable,
            collider: base_class_stats.collider(db_model.appearance.gender),
            collision_layers: Layer::character(),
            base_class,
            vitals_stats,
            primal_stats,
            attack_stats: AttackStats::default(),
            defence_stats: DefenceStats::default(),
            critical_stats: CriticalStats::default(),
            inventory_stats: InventoryStats::default(),
            progress_stats,
            progress_level,
            other_stats,
            stat_modifiers,
            pvp: PvpStats::default(),
            sub_class,
            race: db_model.race,
            appearance: db_model.appearance,
            transform: Transform::from_translation(position.into()),
            last_known_pos: LastKnownPosition {
                position: position.into(),
                timestamp: 0.0,
            },
            paper_doll,
            delete_timer: super::DeleteTimer::default(),
            visibility: EncountersVisibility::default(),
            skill_list,
            known_entities: KnownEntities::default(),
            inventory: Inventory::default(),
            wait_kind: WaitKind::default(),
            attack_effects: AttackEffects::default(),
            defence_effects: DefenceEffects::default(),
            abnormal_effects: AbnormalEffects::default(),
            targetable: Targetable,
        }
    }

    pub fn update(&mut self, character: &character::query::QueryItem) {
        self.name = character.name.clone();
        self.title = character.title.clone();
        self.transform = *character.transform;
        self.race = *character.race;
        self.sub_class = *character.sub_class;
        self.appearance = *character.appearance;
        self.base_class = *character.base_class;
        self.collider = character.collider.clone();
        self.visibility = *character.visibility;
        self.pvp = *character.pvp_stats;
        self.progress_level = character.progress_level.clone();
        self.progress_stats = character.progress_stats.clone();
        self.primal_stats = character.primal_stats.clone();
        self.vitals_stats = character.vitals_stats.clone();
        self.paper_doll = character.paperdoll.clone();
    }
}
