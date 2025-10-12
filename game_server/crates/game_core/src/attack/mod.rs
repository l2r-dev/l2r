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
            .register_type::<AttackingList>()
            .register_type::<AttackTimer>()
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

#[derive(Component, Reflect)]
pub struct AttackHit {
    target: Entity,
    damage: f32,
    critical: bool,
    timer: Timer,
    weapon_entity: Option<Entity>,
}
impl AttackHit {
    pub fn new(
        target: Entity,
        damage: f32,
        critical: bool,
        duration: Duration,
        weapon_entity: Option<Entity>,
    ) -> Self {
        let timer = Timer::new(duration, TimerMode::Once);
        Self {
            target,
            damage,
            critical,
            timer,
            weapon_entity,
        }
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn critical(&self) -> bool {
        self.critical
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
