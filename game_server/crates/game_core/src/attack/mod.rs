use crate::{
    animation::Animation, items::PaperDoll, movement::MoveToEntity, object_id::ObjectId, stats::*,
};
use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};
use std::time::Duration;

mod attacking;
mod death;
mod in_combat;

pub use attacking::*;
pub use death::*;
pub use in_combat::*;

pub struct AttackComponentsPlugin;
impl Plugin for AttackComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Attackable>()
            .register_type::<Attacking>()
            .register_type::<AttackAllowed>()
            .register_type::<AttackHit>()
            .register_type::<AttackCommonHit>()
            .register_type::<AttackMultiHit>()
            .register_type::<AttackingList>()
            .register_type::<AttackTimer>()
            .register_type::<ConsumeArrow>()
            .register_type::<HitInfo>()
            .register_type::<InCombat>();
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Attackable;

#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct AttackAllowed;

#[derive(QueryData)]
#[query_data(mutable)]
struct EnemyQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

#[derive(Component, Reflect, Copy, Clone)]
pub struct HitInfo {
    pub miss: bool,
    pub crit: bool,
    pub shield: ShieldResult,
    pub dmg: f32,
}

#[derive(Component, Default, Reflect)]
pub struct ConsumeArrow;

#[derive(Component, Reflect)]
pub enum AttackHit {
    AttackCommonHit(AttackCommonHit),
    AttackDualHit(AttackDualHit),
    AttackMultiHit(AttackMultiHit),
}

impl AttackHit {
    pub fn timer(&self) -> &Timer {
        match self {
            AttackHit::AttackCommonHit(v) => v.timer(),
            AttackHit::AttackDualHit(v) => v.timer(),
            AttackHit::AttackMultiHit(v) => v.timer(),
        }
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        match self {
            AttackHit::AttackCommonHit(v) => v.timer_mut(),
            AttackHit::AttackDualHit(v) => v.timer_mut(),
            AttackHit::AttackMultiHit(v) => v.timer_mut(),
        }
    }

    pub fn new_common(
        target: Entity,
        duration: Duration,
        hit_info: HitInfo,
        weapon_entity: Option<Entity>,
    ) -> Self {
        Self::AttackCommonHit(AttackCommonHit::new(
            target,
            duration,
            hit_info,
            weapon_entity,
        ))
    }

    pub fn new_dual(
        target: Entity,
        weapon_entity: Option<Entity>,

        first_duration: Duration,
        first_info: HitInfo,

        second_duration: Duration,
        second_info: HitInfo,
    ) -> Self {
        Self::AttackDualHit(AttackDualHit::new(
            target,
            weapon_entity,
            first_duration,
            first_info,
            second_duration,
            second_info,
        ))
    }

    pub fn new_multi(
        duration: Duration,
        weapon_entity: Option<Entity>,
        hits: Vec<(Entity, HitInfo)>,
    ) -> Self {
        Self::AttackMultiHit(AttackMultiHit::new(duration, weapon_entity, hits))
    }
}

#[derive(Component, Reflect)]
pub struct AttackDualHit {
    target: Entity,
    weapon_entity: Option<Entity>,

    primary_timer: Timer,
    primary_info: HitInfo,

    secondary_timer: Timer,
    secondary_info: HitInfo,

    is_primary: bool,
}

impl AttackDualHit {
    fn new(
        target: Entity,
        weapon_entity: Option<Entity>,

        first_duration: Duration,
        first_info: HitInfo,

        second_duration: Duration,
        second_info: HitInfo,
    ) -> Self {
        let first_timer = Timer::new(first_duration, TimerMode::Once);
        let second_timer = Timer::new(second_duration, TimerMode::Once);

        Self {
            target,
            weapon_entity,
            primary_timer: first_timer,
            primary_info: first_info,
            secondary_timer: second_timer,
            secondary_info: second_info,
            is_primary: true,
        }
    }

    fn timer(&self) -> &Timer {
        if self.is_primary {
            &self.primary_timer
        } else {
            &self.secondary_timer
        }
    }

    fn timer_mut(&mut self) -> &mut Timer {
        if self.is_primary {
            &mut self.primary_timer
        } else {
            &mut self.secondary_timer
        }
    }

    pub fn weapon_entity(&self) -> Option<Entity> {
        self.weapon_entity
    }

    pub fn hit(&self) -> HitInfo {
        if self.is_primary {
            self.primary_info
        } else {
            self.secondary_info
        }
    }

    pub fn target(&self) -> Entity {
        self.target
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub fn set_to_secondary(&mut self) -> bool {
        if self.is_primary {
            self.is_primary = false;

            return true;
        }

        false
    }
}

#[derive(Component, Reflect)]
pub struct AttackMultiHit {
    hits: Vec<(Entity, HitInfo)>,
    timer: Timer,
    weapon_entity: Option<Entity>,
}

impl AttackMultiHit {
    fn new(
        duration: Duration,
        weapon_entity: Option<Entity>,
        hits: Vec<(Entity, HitInfo)>,
    ) -> Self {
        let timer = Timer::new(duration, TimerMode::Once);

        Self {
            timer,
            hits,
            weapon_entity,
        }
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn weapon_entity(&self) -> Option<Entity> {
        self.weapon_entity
    }

    pub fn hits(&self) -> &[(Entity, HitInfo)] {
        self.hits.as_slice()
    }
}

#[derive(Component, Reflect)]
pub struct AttackCommonHit {
    target: Entity,
    timer: Timer,
    hit_info: HitInfo,
    weapon_entity: Option<Entity>,
}

impl AttackCommonHit {
    fn new(
        target: Entity,
        duration: Duration,
        hit_info: HitInfo,
        weapon_entity: Option<Entity>,
    ) -> Self {
        let timer = Timer::new(duration, TimerMode::Once);

        Self {
            target,
            timer,
            hit_info,
            weapon_entity,
        }
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn hit(&self) -> HitInfo {
        self.hit_info
    }

    pub fn target(&self) -> Entity {
        self.target
    }

    pub fn weapon_entity(&self) -> Option<Entity> {
        self.weapon_entity
    }
}

#[derive(QueryData)]
struct AttackingQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    attack_stats: Ref<'a, AttackStats>,
    transform: Ref<'a, Transform>,
    target: Ref<'a, Attacking>,
    paper_doll: Option<Ref<'a, PaperDoll>>,
    move_to: Option<Ref<'a, MoveToEntity>>,
}

#[derive(QueryFilter)]
struct AttackingFilter {
    attack_allowed: With<AttackAllowed>,
    not_dead: Without<Dead>,
    // without_pending_skill: Without<PendingSkill>,
    not_animating: Without<Animation>,
}

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct AttackTimer(Timer);
impl AttackTimer {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }

    pub fn timer(&self) -> &Timer {
        &self.0
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.0
    }
}
