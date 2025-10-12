use bevy::prelude::*;
use game_core::custom_hierarchy::CustomHierarchyComponentsPlugin;

pub struct CustomHierarchyPlugin;
impl Plugin for CustomHierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CustomHierarchyComponentsPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::custom_hierarchy::{DespawnChildOf, DespawnChildren};

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
