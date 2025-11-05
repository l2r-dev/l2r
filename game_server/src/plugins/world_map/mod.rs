use bevy::prelude::*;
use map::WorldMapComponentsPlugin;

#[cfg(feature = "gui")]
pub mod gui;

mod path_finding;
mod region;
mod scripting;
mod spawner;
mod zones;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldMapComponentsPlugin);

        app.add_plugins(spawner::SpawnerPlugin)
            .add_plugins(region::RegionPlugin)
            .add_plugins(zones::ZonesPlugin)
            .add_plugins(path_finding::PathFindingPlugin)
            .add_plugins(scripting::RegionGeoDataScriptingPlugin);

        #[cfg(feature = "gui")]
        app.add_plugins(gui::WorldMapGui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::serial;
    use game_core::{character::Character, path_finding::DirectMoveRequest};
    use spatial::GameVec3;

    #[test]
    #[serial]
    fn test_path_finding_and_movement() {
        let mut app = crate::tests::create_test_app();

        let final_pos = GameVec3::new(28423, 11655, -4232).into();

        {
            let world = app.world_mut();
            let mut query = world.query_filtered::<(Entity, Ref<Transform>), With<Character>>();
            let (character_entity, character_transform) = query.single(world).unwrap();

            world.trigger_targets(
                DirectMoveRequest {
                    entity: character_entity,
                    start: character_transform.translation,
                    target: final_pos,
                },
                character_entity,
            );
        }

        const MAX_ITERATIONS: usize = 50000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            app.update();

            let character_pos = {
                let world = app.world_mut();
                let mut query = world.query_filtered::<&Transform, With<Character>>();
                query.single(world).unwrap().translation
            };

            if (character_pos.x - final_pos.x).abs() <= 5.0
                && (character_pos.y - final_pos.y).abs() <= 5.0
                && (character_pos.z - final_pos.z).abs() <= 5.0
            {
                break;
            }

            if iterations >= MAX_ITERATIONS {
                panic!(
                    "Character position {:?} differs from expected {:?} after {} iterations",
                    character_pos, final_pos, MAX_ITERATIONS
                );
            }
        }

        let world = app.world_mut();
        let mut query = world.query_filtered::<&Transform, With<Character>>();
        let character_transform = query.single(world).unwrap().translation;

        assert!(
            (character_transform.x - final_pos.x).abs() <= 5.0
                && (character_transform.y - final_pos.y).abs() <= 5.0
                && (character_transform.z - final_pos.z).abs() <= 5.0,
            "Character position {:?} differs from expected {:?} by more than 5 units",
            character_transform,
            final_pos
        );
    }
}
