use crate::{
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{GameServerPacket, SelectTarget, TargetUnselected},
    },
    object_id::ObjectId,
};
use bevy::{
    ecs::{
        component::{ComponentHook, Immutable, StorageType},
        relationship::{Relationship, RelationshipHookMode, RelationshipSourceCollection},
        world::DeferredWorld,
    },
    prelude::*,
};
use smallvec::SmallVec;

pub struct TargetComponentsPlugin;
impl Plugin for TargetComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SelectedTarget>()
            .register_type::<TargetedBy>();
    }
}

const TARGETED_BY_CAPACITY: usize = 10;

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = SelectedTarget)]
pub struct TargetedBy(SmallVec<[Entity; TARGETED_BY_CAPACITY]>);

#[derive(Clone, Copy, Debug, Deref, DerefMut, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SelectedTarget(pub Entity);

impl Relationship for SelectedTarget {
    type RelationshipTarget = TargetedBy;

    #[inline(always)]
    fn get(&self) -> Entity {
        self.0
    }

    #[inline]
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Component for SelectedTarget {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Immutable;

    fn on_insert() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            match ctx.relationship_hook_mode {
                RelationshipHookMode::Run => {}
                _ => return,
            }
            let Some(selected_target) = world.entity(ctx.entity).get::<Self>() else {
                return;
            };
            let target_entity = selected_target.get();

            if let Ok(mut target_entity_mut) = world.get_entity_mut(target_entity) {
                if let Some(mut relationship_target) =
                    target_entity_mut.get_mut::<<Self as Relationship>::RelationshipTarget>()
                {
                    RelationshipSourceCollection::add(
                        relationship_target.collection_mut_risky(),
                        ctx.entity,
                    );
                } else {
                    let mut target = <<Self as Relationship>::RelationshipTarget>::with_capacity(1);
                    RelationshipSourceCollection::add(target.collection_mut_risky(), ctx.entity);
                    world.commands().entity(target_entity).insert(target);
                }

                if let Some(target_object_id) = world.get::<ObjectId>(target_entity).copied() {
                    world.commands().trigger_targets(
                        GameServerPacket::from(SelectTarget::new(target_object_id, 0)),
                        ctx.entity,
                    );
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

            let Some(selected_target) = world.entity(ctx.entity).get::<Self>() else {
                return;
            };
            let target_entity = selected_target.get();
            let object_id = world.get::<ObjectId>(ctx.entity).copied();
            let transform = world.get::<Transform>(ctx.entity).copied();

            if let Ok(mut target_entity_mut) = world.get_entity_mut(target_entity)
                && let Some(mut relationship_target) =
                    target_entity_mut.get_mut::<<Self as Relationship>::RelationshipTarget>()
            {
                RelationshipSourceCollection::remove(
                    relationship_target.collection_mut_risky(),
                    ctx.entity,
                );

                let is_empty = relationship_target.is_empty();

                // Only remove component if the target is not the same entity
                // This prevents removing from self when switching from self-target to another target
                if is_empty && target_entity != ctx.entity {
                    world.commands().entity(target_entity).remove::<Self>();
                }

                if let (Some(object_id), Some(transform)) = (object_id, transform) {
                    let sending_packet =
                        TargetUnselected::new(object_id, transform.translation).into();

                    world
                        .commands()
                        .trigger_targets(ServerPacketBroadcast::new(sending_packet), ctx.entity);
                }
            }
        })
    }
}
