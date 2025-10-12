#[cfg(test)]
pub mod test_utils {

    use crate::tests::{create_test_app, get_character_entity_and_oid};
    use bevy::prelude::*;
    use game_core::{
        items::{
            EquipItems, Id, Inventory, Item, ItemLocation, ItemsDataTable, ItemsInfo, PaperDoll,
            SpawnExisting,
        },
        object_id::{ObjectId, ObjectIdManager, QueryStateByObjectId},
    };

    // 17 - Wooden Arrow, 57 - Adena
    pub const STACKABLE_ITEMS: [Id; 2] = [Id::new(17), Id::new(57)];
    // 1 - Short Sword, 25 - Piece Bone Breastplate
    pub const NON_STACKABLE_ITEMS: [Id; 2] = [Id::new(1), Id::new(25)];

    /// Creates a test app with items data loaded
    pub fn create_test_app_with_items_data() -> App {
        let mut app = create_test_app();

        let world = app.world_mut();
        let asset_server = world.resource::<AssetServer>();
        world.insert_resource(ItemsDataTable::test_data(asset_server));

        let mut items_iterations = 0;
        const MAX_ITEMS_ITERATIONS: u32 = 5000;

        loop {
            items_iterations += 1;
            app.update();
            let world = app.world_mut();
            let items_assets = world.resource::<Assets<ItemsInfo>>();
            if !items_assets.is_empty() {
                println!("Items assets loaded after {} iterations", items_iterations);
                break;
            }

            if items_iterations >= MAX_ITEMS_ITERATIONS {
                println!(
                    "Warning: Items assets loading timeout after {} iterations",
                    items_iterations
                );
                break; // Continue without items assets for tests
            }
        }

        app
    }

    /// Sets up an app with items in the character's inventory
    pub fn setup_app_with_items_in_inventory() -> (App, Entity, Vec<ObjectId>, Vec<ObjectId>) {
        let mut app = create_test_app_with_items_data();

        let (character_entity, character_oid) = get_character_entity_and_oid(&mut app);

        let non_stackable_oids = NON_STACKABLE_ITEMS
            .iter()
            .map(|&item_id| add_item(&mut app, character_entity, character_oid, item_id, 1))
            .collect::<Vec<_>>();

        let stackable_oids = STACKABLE_ITEMS
            .iter()
            .map(|&item_id| add_item(&mut app, character_entity, character_oid, item_id, 150))
            .collect::<Vec<_>>();

        (app, character_entity, stackable_oids, non_stackable_oids)
    }

    /// Sets up an app with equipped items
    pub fn setup_app_with_equipped_items() -> (App, Entity, Vec<ObjectId>) {
        let (mut app, character_entity, _stackable_oids, non_stackable_oids) =
            setup_app_with_items_in_inventory();

        let world = app.world_mut();

        world.send_event(EquipItems::new(
            character_entity,
            non_stackable_oids.clone(),
        ));

        app.update();

        (app, character_entity, non_stackable_oids)
    }

    /// Adds an item to a character's inventory
    pub fn add_item(
        app: &mut App,
        character_entity: Entity,
        character_oid: ObjectId,
        item_id: Id,
        count: u64,
    ) -> ObjectId {
        use game_core::items::model::Model;

        let object_id = {
            let world = app.world_mut();
            let new_oid = {
                let mut object_id_manager = world.resource_mut::<ObjectIdManager>();
                object_id_manager.next_id()
            };

            let new_item = Model::new(
                new_oid,
                item_id,
                count,
                ItemLocation::Inventory,
                Some(character_oid),
            );

            world.trigger_targets(
                SpawnExisting {
                    item_models: vec![new_item],
                    dropped_entity: None,
                    silent: true,
                },
                character_entity,
            );

            new_oid
        };

        app.update();

        object_id
    }

    /// Verifies stacking behavior with expected total count
    pub fn verify_stacking_behavior_with_total(
        app: &mut App,
        character_entity: Entity,
        original_oid: ObjectId,
        additional_oids: Vec<ObjectId>,
        expected_total: u64,
    ) {
        let world = app.world_mut();
        let mut stackable_item_query = world.query_filtered::<Entity, With<Item>>();

        let object_id_manager = world.resource::<ObjectIdManager>();
        for &oid in &additional_oids {
            assert!(
                stackable_item_query
                    .by_object_id(world, oid, object_id_manager)
                    .is_err(),
                "Additional stackable item should be despawned"
            );
            assert!(
                object_id_manager.entity(oid).is_none(),
                "Object ID should be released for the additional item"
            );
        }

        let stackable_item_entity = {
            let object_id_manager = world.resource::<ObjectIdManager>();
            stackable_item_query
                .by_object_id(world, original_oid, object_id_manager)
                .unwrap()
        };

        let item = app
            .world()
            .entity(stackable_item_entity)
            .get::<Item>()
            .unwrap();
        assert_eq!(
            item.count(),
            expected_total,
            "Original stackable item should now have combined count"
        );

        let inventory = app
            .world()
            .entity(character_entity)
            .get::<Inventory>()
            .unwrap();
        assert!(
            inventory.contains(&original_oid),
            "Original stackable item should still be in inventory"
        );
    }

    /// Verifies items are equipped or unequipped in the paperdoll
    pub fn verify_item_equipment_status(
        app: &mut App,
        character_entity: Entity,
        item_oids: &[ObjectId],
        should_be_equipped: bool,
    ) {
        let world = app.world();

        // Verify PaperDoll component has or does not have the items
        let paperdoll = world
            .entity(character_entity)
            .get::<PaperDoll>()
            .expect("Character should have PaperDoll component");

        for &oid in item_oids {
            assert_eq!(
                paperdoll.is_equipped(oid),
                should_be_equipped,
                "Item {:?} equip status in PaperDoll was not as expected. Expected: {}, Actual: {}",
                oid,
                should_be_equipped,
                paperdoll.is_equipped(oid)
            );
        }

        // Verify each item's location
        let object_id_manager = world.resource::<ObjectIdManager>();

        for &oid in item_oids {
            if let Some(item_entity) = object_id_manager.entity(oid) {
                let item = world
                    .entity(item_entity)
                    .get::<Item>()
                    .expect("Item component should exist");

                match item.location() {
                    ItemLocation::PaperDoll(_) => {
                        if !should_be_equipped {
                            panic!(
                                "Item {:?} should NOT be in PaperDoll location (expected unequipped), but found it there.",
                                oid
                            );
                        }
                    }
                    _ => {
                        // Not in PaperDoll (e.g., Inventory, World)
                        if should_be_equipped {
                            panic!(
                                "Item {:?} should be in PaperDoll location (expected equipped), but found: {:?}",
                                oid,
                                item.location()
                            );
                        }
                    }
                }
            } else if should_be_equipped {
                // If we expect it to be equipped, it must exist.
                panic!(
                    "Item {:?} entity should exist if it's expected to be equipped, but it was not found.",
                    oid
                );
            }
        }
    }

    pub fn verify_items_in_inventory(
        app: &mut App,
        character_entity: Entity,
        expected_inventory_oids: &[ObjectId],
        should_exist: bool,
    ) {
        let world = app.world();

        // Get the Inventory component of the character
        let inventory = world
            .entity(character_entity)
            .get::<Inventory>()
            .expect("Character should have Inventory component");

        // Verify each expected item is in the inventory (or not)
        for &expected_oid in expected_inventory_oids {
            if should_exist {
                assert!(
                    inventory.contains(&expected_oid),
                    "Expected item {:?} should be in inventory",
                    expected_oid
                );
            } else {
                assert!(
                    !inventory.contains(&expected_oid),
                    "Item {:?} should NOT be in inventory",
                    expected_oid
                );
            }
        }

        let object_id_manager = world.resource::<ObjectIdManager>();

        // Verify each item's location is Inventory (only if should_exist is true)
        if should_exist {
            for &expected_oid in expected_inventory_oids {
                if let Some(item_entity) = object_id_manager.entity(expected_oid) {
                    let item = world
                        .entity(item_entity)
                        .get::<Item>()
                        .expect("Item component should exist");

                    match item.location() {
                        ItemLocation::Inventory => {} // Expected
                        other_location => {
                            panic!(
                                "Item {:?} should be in Inventory location, but found: {:?}",
                                expected_oid, other_location
                            );
                        }
                    }
                }
            }
        }
    }
}

mod added_tests {
    use super::test_utils::*;
    use crate::tests::serial;
    use game_core::object_id::ObjectId;

    #[test]
    #[serial]
    fn test_adding_items_to_inventory() {
        let (mut app, character_entity, stackable_oids, non_stackable_oids) =
            setup_app_with_items_in_inventory();

        verify_items_in_inventory(&mut app, character_entity, &stackable_oids, true);
        verify_items_in_inventory(&mut app, character_entity, &non_stackable_oids, true);
    }

    #[test]
    #[serial]
    fn test_stacking_behavior() {
        let (mut app, character_entity, stackable_oids, _non_stackable_oids) =
            setup_app_with_items_in_inventory();

        // Get character_oid for adding additional items
        let character_oid = {
            let world = app.world();
            *world.entity(character_entity).get::<ObjectId>().unwrap()
        };

        let additional_stackable_oids = STACKABLE_ITEMS
            .iter()
            .map(|&item_id| add_item(&mut app, character_entity, character_oid, item_id, 5))
            .collect::<Vec<_>>();

        let additional2_stackable_oids = STACKABLE_ITEMS
            .iter()
            .map(|&item_id| add_item(&mut app, character_entity, character_oid, item_id, 10))
            .collect::<Vec<_>>();

        for i in 0..STACKABLE_ITEMS.len() {
            verify_stacking_behavior_with_total(
                &mut app,
                character_entity,
                stackable_oids[i],
                vec![additional_stackable_oids[i], additional2_stackable_oids[i]],
                165,
            );
        }
    }
}

#[cfg(test)]
mod equip_tests {
    use super::test_utils::*;
    use crate::tests::serial;

    #[test]
    #[serial]
    fn test_equip_items_from_inventory() {
        let (mut app, character_entity, non_stackable_oids) = setup_app_with_equipped_items();

        verify_item_equipment_status(&mut app, character_entity, &non_stackable_oids, true);
    }
}

#[cfg(test)]
mod unequip_tests {
    use super::test_utils::*;
    use crate::tests::serial;
    use game_core::items::UnequipItems;

    #[test]
    #[serial]
    fn test_unequip_items() {
        let (mut app, character_entity, equipped_oids) = setup_app_with_equipped_items();

        let world = app.world_mut();

        world.send_event(UnequipItems::new(character_entity, equipped_oids.clone()));

        app.update();

        verify_item_equipment_status(&mut app, character_entity, &equipped_oids, false);
        verify_items_in_inventory(&mut app, character_entity, &equipped_oids, true);
    }
}

#[cfg(test)]
mod paperdoll_tests {
    use game_core::{
        items::{BodyPart, DollSlot, Item, ItemInfo, ItemLocation, PaperDoll, UniqueItem},
        object_id::ObjectId,
    };

    #[test]
    fn test_set_and_get() {
        let mut paperdoll = PaperDoll::default();

        let default_item_info = ItemInfo::default();

        let item1 = UniqueItem::new(
            ObjectId::from(5001),
            Item::new(
                1001.into(),
                ItemLocation::PaperDoll(DollSlot::Head),
                &default_item_info,
            ),
        );
        let item2 = UniqueItem::new(
            ObjectId::from(5002),
            Item::new(
                1002.into(),
                ItemLocation::PaperDoll(DollSlot::Chest),
                &default_item_info,
            ),
        );
        let item3 = UniqueItem::new(
            ObjectId::from(5003),
            Item::new(
                1003.into(),
                ItemLocation::PaperDoll(DollSlot::Legs),
                &default_item_info,
            ),
        );

        // Setting and getting items in different slots
        paperdoll.equip(BodyPart::Head, Some(item1));
        paperdoll.equip(BodyPart::Chest, Some(item2));
        paperdoll.equip(BodyPart::Legs, Some(item3));

        assert_eq!(paperdoll.get(DollSlot::Head), Some(item1));
        assert_eq!(paperdoll.get(DollSlot::Chest), Some(item2));
        assert_eq!(paperdoll.get(DollSlot::Legs), Some(item3));

        // Setting slot to None
        paperdoll.equip(BodyPart::Head, None);
        assert_eq!(paperdoll.get(DollSlot::Head), None);
    }
}

#[cfg(test)]
mod drop_tests {
    use super::test_utils::*;
    use crate::tests::serial;
    use bevy::{ecs::relationship::Relationship, prelude::*};
    use game_core::{
        custom_hierarchy::DespawnChildOf,
        items::{DropIfPossible, Item, ItemLocation},
        object_id::{ObjectId, ObjectIdManager},
    };
    use map::{WorldMap, id::RegionId};

    const DROP_POSITION: Vec3 = Vec3::new(28300.0, -4224.0, 11070.0);
    const FAR_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    #[test]
    #[serial]
    fn test_drop_unequipped_non_stackable_item() {
        let (mut app, character_entity, _stackable_oids, non_stackable_oids) =
            setup_app_with_items_in_inventory();

        let item_to_drop = non_stackable_oids[0];

        app.world_mut().trigger_targets(
            DropIfPossible {
                item_oid: item_to_drop,
                count: 1,
                location: DROP_POSITION,
            },
            character_entity,
        );

        app.update();

        verify_items_in_inventory(&mut app, character_entity, &[item_to_drop], false);

        // Verify item exists in world
        let world = app.world();
        let object_id_manager = world.resource::<ObjectIdManager>();
        let item_entity = object_id_manager
            .entity(item_to_drop)
            .expect("Dropped item should still exist");

        // Verify item location is World
        let item = world.entity(item_entity).get::<Item>().unwrap();
        assert!(
            matches!(item.location(), ItemLocation::World(_)),
            "Dropped item should have World location"
        );

        // Verify item has correct position
        let transform = world.entity(item_entity).get::<Transform>().unwrap();
        assert_eq!(
            transform.translation, DROP_POSITION,
            "Dropped item should be at drop position"
        );

        let region_id = RegionId::from(DROP_POSITION);
        if let Some(region_entity) = world.resource::<WorldMap>().get(&region_id) {
            let child_of = world.entity(item_entity).get::<DespawnChildOf>().unwrap();
            assert_eq!(child_of.get(), *region_entity);
        }
    }

    #[test]
    #[serial]
    fn test_cant_drop_item_in_invalid_location() {
        let (mut app, character_entity, _stackable_oids, non_stackable_oids) =
            setup_app_with_items_in_inventory();

        let item_to_drop = non_stackable_oids[0];

        // Attempt to drop item in an invalid location
        app.world_mut().trigger_targets(
            DropIfPossible {
                item_oid: item_to_drop,
                count: 1,
                location: FAR_POSITION,
            },
            character_entity,
        );

        app.update();

        // Verify item is still in inventory
        verify_items_in_inventory(&mut app, character_entity, &[item_to_drop], true);
    }

    #[test]
    #[serial]
    fn test_drop_equipped_item() {
        let (mut app, character_entity, equipped_oids) = setup_app_with_equipped_items();

        let item_to_drop = equipped_oids[0];

        // Verify item is equipped before dropping
        verify_item_equipment_status(&mut app, character_entity, &[item_to_drop], true);

        // Drop the equipped item
        app.world_mut().trigger_targets(
            DropIfPossible {
                item_oid: item_to_drop,
                count: 1,
                location: DROP_POSITION,
            },
            character_entity,
        );

        app.update();

        // Verify item is no longer equipped
        verify_item_equipment_status(&mut app, character_entity, &[item_to_drop], false);

        // Verify item is no longer in inventory
        verify_items_in_inventory(&mut app, character_entity, &[item_to_drop], false);

        // Verify item exists in world
        let world = app.world();
        let object_id_manager = world.resource::<ObjectIdManager>();
        let item_entity = object_id_manager
            .entity(item_to_drop)
            .expect("Dropped item should still exist");

        let item = world.entity(item_entity).get::<Item>().unwrap();
        assert!(
            matches!(item.location(), ItemLocation::World(_)),
            "Dropped item should have World location"
        );
    }

    #[test]
    #[serial]
    fn test_drop_full_stackable_item() {
        let (mut app, character_entity, stackable_oids, _non_stackable_oids) =
            setup_app_with_items_in_inventory();

        let item_to_drop = stackable_oids[0];
        let original_count = 150;

        // Drop the entire stack
        app.world_mut().trigger_targets(
            DropIfPossible {
                item_oid: item_to_drop,
                count: original_count,
                location: DROP_POSITION,
            },
            character_entity,
        );

        app.update();

        // Verify item is no longer in inventory
        verify_items_in_inventory(&mut app, character_entity, &[item_to_drop], false);

        // Verify item exists in world with correct count
        let world = app.world();
        let object_id_manager = world.resource::<ObjectIdManager>();
        let item_entity = object_id_manager
            .entity(item_to_drop)
            .expect("Dropped item should still exist");

        let item = world.entity(item_entity).get::<Item>().unwrap();
        assert_eq!(
            item.count(),
            original_count,
            "Dropped item should have full count"
        );
        assert!(
            matches!(item.location(), ItemLocation::World(_)),
            "Dropped item should have World location"
        );
    }

    #[test]
    #[serial]
    fn test_drop_partial_stackable_item() {
        let (mut app, character_entity, stackable_oids, _non_stackable_oids) =
            setup_app_with_items_in_inventory();

        let item_to_drop = stackable_oids[0];
        let original_count = 150;
        let drop_count = 50;
        let remaining_count = original_count - drop_count;

        // Drop part of the stack
        app.world_mut().trigger_targets(
            DropIfPossible {
                item_oid: item_to_drop,
                count: drop_count,
                location: DROP_POSITION,
            },
            character_entity,
        );

        app.update();

        // Verify original item is still in inventory with reduced count
        verify_items_in_inventory(&mut app, character_entity, &[item_to_drop], true);

        let original_item_entity = {
            let world = app.world_mut();
            let object_id_manager = world.resource::<ObjectIdManager>();
            object_id_manager
                .entity(item_to_drop)
                .expect("Original item should still exist")
        };

        let world = app.world_mut();

        let original_item = *world
            .entity(original_item_entity)
            .get::<Item>()
            .expect("Original item should have Item component");

        assert_eq!(
            original_item.count(),
            remaining_count,
            "Original item should have reduced count"
        );
        assert!(
            matches!(original_item.location(), ItemLocation::Inventory),
            "Original item should still be in inventory"
        );

        // Find the new dropped item (it will have a different ObjectId)
        let mut dropped_item_found = false;
        let mut item_query = world.query_filtered::<(Entity, &Item, &ObjectId), With<Item>>();

        for (entity, item, object_id) in item_query.iter(world) {
            if matches!(item.location(), ItemLocation::World(_)) && item.count() == drop_count {
                dropped_item_found = true;

                // Verify the dropped item properties
                assert_eq!(
                    item.id(),
                    original_item.id(),
                    "Dropped item should have same item ID"
                );

                let transform = world.entity(entity).get::<Transform>().unwrap();
                assert_eq!(
                    transform.translation, DROP_POSITION,
                    "Dropped item should be at drop position"
                );

                // Verify it's registered in the object ID manager
                let object_id_manager = world.resource::<ObjectIdManager>();
                assert!(
                    object_id_manager.entity(*object_id).is_some(),
                    "Dropped item should be registered in ObjectIdManager"
                );

                break;
            }
        }

        assert!(
            dropped_item_found,
            "Should find a dropped item with count {} in the world",
            drop_count
        );
    }
}
