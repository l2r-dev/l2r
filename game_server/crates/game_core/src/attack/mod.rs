use crate::{object_id::ObjectId, stats::*};
use bevy::{ecs::query::QueryData, prelude::*};
use std::time::Duration;

mod attacking;
mod death;
mod in_combat;

use crate::items::Grade;
pub use attacking::*;
pub use death::*;
pub use in_combat::*;

pub struct AttackComponentsPlugin;
impl Plugin for AttackComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Attackable>()
            .register_type::<Attacking>()
            .register_type::<AttackHit>()
            .register_type::<AttackCommonHit>()
            .register_type::<AttackMultiHit>()
            .register_type::<AttackingList>()
            .register_type::<ConsumeArrow>()
            .register_type::<HitInfo>()
            .register_type::<InCombat>()
            .register_type::<Immortal>()
            .register_type::<WeaponReuse>();
    }
}

#[derive(Clone, Component, Copy, Debug, Default, Reflect)]
pub struct Attackable;

#[derive(QueryData)]
#[query_data(mutable)]
struct EnemyQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

#[derive(Component, Default, Reflect)]
#[component(storage = "SparseSet")]
pub struct Immortal;

#[derive(Default, Event, Reflect)]
pub struct ConsumeArrow;

#[derive(Component, Default, Reflect)]
#[component(storage = "SparseSet")]
pub struct WeaponReuse {
    timer: Timer,
}

impl WeaponReuse {
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    ///Returns true if finished
    pub fn proceed_timer(&mut self, dt: Duration) -> bool {
        self.timer.tick(dt);

        self.timer.finished()
    }

    pub fn secs_left(&self) -> f32 {
        self.timer.remaining().as_secs_f32()
    }
}

#[derive(Clone, Component, Copy, Reflect)]
pub struct HitInfo {
    pub ss_grade: Option<Grade>,
    pub miss: bool,
    pub crit: bool,
    pub shield: ShieldResult,
    pub damage: f32,
}

#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub enum AttackHit {
    AttackCommonHit(AttackCommonHit),
    AttackDualHit(AttackDualHit),
    AttackMultiHit(AttackMultiHit),
}

impl AttackHit {
    pub fn timer(&self) -> &Timer {
        match self {
            AttackHit::AttackCommonHit(hit) => hit.timer(),
            AttackHit::AttackDualHit(hit) => hit.timer(),
            AttackHit::AttackMultiHit(hit) => hit.timer(),
        }
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        match self {
            AttackHit::AttackCommonHit(hit) => hit.timer_mut(),
            AttackHit::AttackDualHit(hit) => hit.timer_mut(),
            AttackHit::AttackMultiHit(hit) => hit.timer_mut(),
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
        both_missed: bool,
    ) -> Self {
        Self::AttackDualHit(AttackDualHit::new(
            target,
            weapon_entity,
            first_duration,
            first_info,
            second_duration,
            second_info,
            both_missed,
        ))
    }

    pub fn new_multi(
        duration: Duration,
        weapon_entity: Option<Entity>,
        hits: Vec<(Entity, HitInfo)>,
        all_missed: bool,
    ) -> Self {
        Self::AttackMultiHit(AttackMultiHit::new(
            duration,
            weapon_entity,
            hits,
            all_missed,
        ))
    }

    pub fn remove_soulshot(&self) -> bool {
        match self {
            AttackHit::AttackCommonHit(hit) => hit.remove_soulshot(),
            AttackHit::AttackDualHit(hit) => hit.remove_soulshot(),
            AttackHit::AttackMultiHit(hit) => hit.remove_soulshot(),
        }
    }

    pub fn weapon_entity(&self) -> Option<Entity> {
        match self {
            AttackHit::AttackCommonHit(hit) => hit.weapon_entity,
            AttackHit::AttackDualHit(hit) => hit.weapon_entity,
            AttackHit::AttackMultiHit(hit) => hit.weapon_entity,
        }
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
    both_missed: bool,
}

impl AttackDualHit {
    fn new(
        target: Entity,
        weapon_entity: Option<Entity>,

        first_duration: Duration,
        first_info: HitInfo,

        second_duration: Duration,
        second_info: HitInfo,

        both_missed: bool,
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
            both_missed,
        }
    }

    fn remove_soulshot(&self) -> bool {
        !self.both_missed && self.is_primary
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
        std::mem::take(&mut self.is_primary)
    }
}

#[derive(Component, Reflect)]
pub struct AttackMultiHit {
    hits: Vec<(Entity, HitInfo)>,
    timer: Timer,
    weapon_entity: Option<Entity>,
    all_missed: bool,
}

impl AttackMultiHit {
    fn new(
        duration: Duration,
        weapon_entity: Option<Entity>,
        hits: Vec<(Entity, HitInfo)>,
        all_missed: bool,
    ) -> Self {
        let timer = Timer::new(duration, TimerMode::Once);

        Self {
            timer,
            hits,
            weapon_entity,
            all_missed,
        }
    }

    fn remove_soulshot(&self) -> bool {
        !self.all_missed
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
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

    fn remove_soulshot(&self) -> bool {
        !self.hit_info.miss
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
}
