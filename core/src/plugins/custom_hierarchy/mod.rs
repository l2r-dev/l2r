use bevy::prelude::*;
use bevy_ecs::component::{ComponentCloneBehavior, ComponentHook, Mutable, StorageType};
use smallvec::SmallVec;
use std::any::TypeId;

pub struct CustomHierarchyPlugin;
impl Plugin for CustomHierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DespawnChildOf>()
            .register_type::<DespawnChildren>();
    }
}

const DESPAWN_CHILDREN_CAPACITY: usize = 10;

#[derive(Clone, Component, Copy, Debug, Deref, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = DespawnChildren)]
pub struct DespawnChildOf(#[entities] pub Entity);

#[derive(Clone, Debug, Deref, Reflect)]
#[reflect(Component)]
pub struct DespawnChildren(SmallVec<[Entity; DESPAWN_CHILDREN_CAPACITY]>);

// Use custom implementation because of on_remove needs same behaviour as on_despawn
impl Component for DespawnChildren
where
    Self: Send + Sync + 'static,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn register_required_components(
        _requiree: bevy_ecs::component::ComponentId,
        components: &mut bevy_ecs::component::ComponentsRegistrator,
        _required_components: &mut bevy_ecs::component::RequiredComponents,
        _inheritance_depth: u16,
        recursion_check_stack: &mut bevy_ecs::__macro_exports::Vec<
            bevy_ecs::component::ComponentId,
        >,
    ) {
        bevy_ecs::component::enforce_no_required_components_recursion(
            components,
            recursion_check_stack,
        );
        let self_id = components.register_component::<Self>();
        recursion_check_stack.push(self_id);
        recursion_check_stack.pop();
    }

    fn on_replace() -> ::core::option::Option<ComponentHook> {
        Some(<Self as RelationshipTarget>::on_replace)
    }
    fn on_despawn() -> ::core::option::Option<ComponentHook> {
        Some(<Self as RelationshipTarget>::on_despawn)
    }
    fn on_remove() -> ::core::option::Option<ComponentHook> {
        Some(<Self as RelationshipTarget>::on_despawn)
    }
    fn clone_behavior() -> ComponentCloneBehavior {
        ComponentCloneBehavior::Custom(bevy_ecs::relationship::clone_relationship_target::<Self>)
    }
    fn map_entities<M: bevy_ecs::entity::EntityMapper>(this: &mut Self, mapper: &mut M) {
        use bevy_ecs::entity::MapEntities;
        this.0.map_entities(mapper);
    }
}

impl RelationshipTarget for DespawnChildren {
    const LINKED_SPAWN: bool = true;
    type Relationship = DespawnChildOf;
    type Collection = SmallVec<[Entity; DESPAWN_CHILDREN_CAPACITY]>;
    #[inline]
    fn collection(&self) -> &Self::Collection {
        &self.0
    }
    #[inline]
    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.0
    }
    #[inline]
    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self(collection)
    }
}

pub trait HierarchyFolderOperations {
    fn get_folder<T: Component>(&self) -> Option<Entity>;
    fn set_folder<T: Component>(&mut self, folder_entity: Entity);
    fn remove_folder<T: Component>(&mut self);
    fn folders_iter(&self) -> impl Iterator<Item = (TypeId, Entity)>;
}

pub trait InsertIntoFolders {
    fn insert_into_folders<T: HierarchyFolderOperations>(
        &mut self,
        child_entity: EntityRef,
        folders: &T,
    );
}

impl InsertIntoFolders for Commands<'_, '_> {
    fn insert_into_folders<T: HierarchyFolderOperations>(
        &mut self,
        child_entity: EntityRef,
        folders: &T,
    ) {
        for (component_type_id, folder_entity) in folders.folders_iter() {
            let contains = child_entity.contains_type_id(component_type_id);

            if contains {
                self.entity(child_entity.id())
                    .insert(DespawnChildOf(folder_entity));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_despawn_child_of_hierarchy_spawns_and_despawns() {
        let mut app = App::new();
        app.add_plugins(CustomHierarchyPlugin);

        // Spawn parent entity
        let parent = app.world_mut().spawn_empty().id();

        // Spawn child entities with DespawnChildOf relationship
        let child1 = app.world_mut().spawn(DespawnChildOf(parent)).id();
        let child2 = app.world_mut().spawn(DespawnChildOf(parent)).id();

        // Spawn grandchild
        let grandchild = app.world_mut().spawn(DespawnChildOf(child1)).id();

        // Verify the hierarchy is established correctly
        {
            let parent_entity = app.world().entity(parent);
            let children = parent_entity.get::<DespawnChildren>().unwrap();
            assert_eq!(children.len(), 2);
            assert!(children.contains(&child1));
            assert!(children.contains(&child2));
        }

        {
            let child1_entity = app.world().entity(child1);
            let children = child1_entity.get::<DespawnChildren>().unwrap();
            assert_eq!(children.len(), 1);
            assert!(children.contains(&grandchild));
        }

        // Remove child2 from hierarchy
        if let Ok(mut child2_entity) = app.world_mut().get_entity_mut(child2) {
            child2_entity.remove::<DespawnChildOf>();
        }

        if let Ok(mut parent_entity) = app.world_mut().get_entity_mut(parent) {
            parent_entity.remove::<DespawnChildren>();
        }

        // Verify parent and remaining children are despawned
        assert!(app.world().get_entity(parent).is_ok());
        assert!(app.world().get_entity(child1).is_err());
        assert!(app.world().get_entity(grandchild).is_err());

        // child2 should still exist since it was removed from the hierarchy
        assert!(app.world().get_entity(child2).is_ok());
    }
}
