use crate::object_id::ObjectId;
use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, Mutable, StorageType},
        world::DeferredWorld,
    },
    log,
    prelude::*,
};
use bevy_ecs::entity::EntityHashSet;

pub struct EncountersComponentsPlugin;
impl Plugin for EncountersComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KnownEntities>()
            .register_type::<EnteredWorld>();

        app.add_event::<KnownAdded>()
            .add_event::<KnownRemoved>()
            .add_event::<KnownEntitiesRemoved>();
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct EnteredWorld;

#[derive(Clone, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct KnownEntities(EntityHashSet);
impl Component for KnownEntities {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            let Some(object_id) = world.entity(context.entity).get::<ObjectId>().copied() else {
                log::error!("KnownEntities component removed from entity without ObjectId");
                return;
            };

            world
                .commands()
                .trigger_targets(KnownEntitiesRemoved::new(object_id), context.entity);
        })
    }
}

impl KnownEntities {
    /// Finds an entity either in the known entities or checks if it's the character itself
    pub fn find_known_or_self(
        &self,
        target_entity: Entity,
        character_entity: Entity,
    ) -> Option<Entity> {
        self.iter()
            .copied()
            .chain(std::iter::once(character_entity))
            .find(|&candidate| candidate == target_entity)
    }
}

#[derive(Debug, Event)]
pub struct KnownAdded(Entity);

impl KnownAdded {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ContainsEntity for KnownAdded {
    fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Event)]
pub struct KnownRemoved(ObjectId);
impl KnownRemoved {
    pub fn new(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    pub fn object_id(&self) -> ObjectId {
        self.0
    }
}

#[derive(Event)]
pub struct KnownEntitiesRemoved(ObjectId);
impl KnownEntitiesRemoved {
    pub fn new(object_id: ObjectId) -> Self {
        Self(object_id)
    }

    pub fn object_id(&self) -> ObjectId {
        self.0
    }
}
