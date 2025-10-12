use bevy::{
    ecs::{
        component::{ComponentHook, Immutable, StorageType},
        relationship::{Relationship, RelationshipHookMode, RelationshipSourceCollection},
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_ecs::entity::MapEntities;
use smallvec::SmallVec;

const ATTACKERS_CAPACITY: usize = 2;

#[derive(Clone, Copy, Debug, Deref, Reflect)]
#[reflect(Component)]
pub struct Attacking(pub Entity);

impl Component for Attacking {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Immutable;

    fn on_insert() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            match ctx.relationship_hook_mode {
                RelationshipHookMode::Run => {}
                _ => return,
            }
            let target_entity = world.entity(ctx.entity).get::<Self>().unwrap().get();
            // Can't Attack self
            if target_entity == ctx.entity {
                world.commands().entity(ctx.entity).remove::<Self>();
                return;
            }
            if let Ok(mut target_entity_mut) = world.get_entity_mut(target_entity) {
                if let Some(mut relationship_target) =
                    target_entity_mut.get_mut::<<Self as Relationship>::RelationshipTarget>()
                {
                    relationship_target.add(ctx.entity);
                } else {
                    let mut target = <<Self as Relationship>::RelationshipTarget>::with_capacity(1);
                    target.add(ctx.entity);
                    world.commands().entity(target_entity).insert(target);
                }
            } else {
                world.commands().entity(ctx.entity).remove::<Self>();
            }
        })
    }

    fn on_replace() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            match ctx.relationship_hook_mode {
                RelationshipHookMode::Run => {}
                _ => return,
            }

            let target_entity = world.entity(ctx.entity).get::<Self>().unwrap().get();
            if let Ok(mut target_entity_mut) = world.get_entity_mut(target_entity)
                && let Some(mut relationship_target) =
                    target_entity_mut.get_mut::<<Self as Relationship>::RelationshipTarget>()
            {
                relationship_target.remove(ctx.entity);
            }
        })
    }
}

impl Relationship for Attacking {
    type RelationshipTarget = AttackingList;

    #[inline(always)]
    fn get(&self) -> Entity {
        self.0
    }

    #[inline]
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = Attacking)]
pub struct AttackingList {
    #[relationship]
    store: AttackingListStore,
    last_attacker: Option<Entity>,
}

impl AttackingList {
    pub fn new() -> Self {
        Self {
            store: AttackingListStore::new(),
            last_attacker: None,
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.store.add(entity);
    }

    pub fn damage(&mut self, entity: Entity, damage: f64) {
        if let Some(existing_damage) = self.store.get_damage_mut(entity) {
            *existing_damage += damage;
            self.last_attacker = Some(entity);
        }
    }

    pub fn last_attacker(&self) -> Option<Entity> {
        self.last_attacker
    }

    pub fn get_damage(&self, entity: Entity) -> Option<&f64> {
        self.store.get_damage(entity)
    }

    pub fn get_attackers(&self) -> impl Iterator<Item = (Entity, &f64)> + '_ {
        self.store.get_attackers_with_damage()
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        let removed = self.store.remove(entity);
        if removed && self.last_attacker == Some(entity) {
            self.last_attacker = None;
        }
        removed
    }

    pub fn get_highest_damage_dealer(&self) -> Option<(Entity, f64)> {
        self.store.get_highest_damage_dealer()
    }
}

impl AttackingListStore {
    pub fn get_damage(&self, entity: Entity) -> Option<&f64> {
        self.0
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, damage)| damage)
    }

    pub fn get_damage_mut(&mut self, entity: Entity) -> Option<&mut f64> {
        self.0
            .iter_mut()
            .find(|(e, _)| *e == entity)
            .map(|(_, damage)| damage)
    }

    pub fn get_attackers_with_damage(&self) -> impl Iterator<Item = (Entity, &f64)> + '_ {
        self.0.iter().map(|(entity, damage)| (*entity, damage))
    }

    pub fn get_highest_damage_dealer(&self) -> Option<(Entity, f64)> {
        self.0
            .iter()
            .enumerate()
            .max_by(|(_, (_, damage_a)), (_, (_, damage_b))| {
                damage_a
                    .partial_cmp(damage_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, (entity, damage))| (*entity, *damage))
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct AttackingListStore(SmallVec<[(Entity, f64); ATTACKERS_CAPACITY]>);

impl RelationshipSourceCollection for AttackingListStore {
    type SourceIter<'a> =
        core::iter::Map<core::slice::Iter<'a, (Entity, f64)>, fn(&(Entity, f64)) -> Entity>;

    fn new() -> Self {
        Self(SmallVec::new())
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    fn add(&mut self, entity: Entity) -> bool {
        self.0.push((entity, 0.0));
        true
    }

    fn remove(&mut self, entity: Entity) -> bool {
        if let Some(index) = self.0.iter().position(|(e, _)| *e == entity) {
            self.0.remove(index);
            return true;
        }
        false
    }

    fn iter(&self) -> Self::SourceIter<'_> {
        self.0.iter().map(|(entity, _)| *entity)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    fn extend_from_iter(&mut self, entities: impl IntoIterator<Item = Entity>) {
        self.0
            .extend(entities.into_iter().map(|entity| (entity, 0.0)));
    }
}

impl MapEntities for AttackingListStore {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (entity, _) in &mut self.0 {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}
