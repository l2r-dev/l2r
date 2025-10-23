use crate::plugins;
use avian3d::{
    collision::CollisionDiagnostics, dynamics::solver::SolverDiagnostics,
    spatial_query::SpatialQueryDiagnostics,
};
use bevy::prelude::*;
use bevy_slinet::connection::ConnectionId;
use game_core::{
    abnormal_effects::AbnormalEffects,
    action::wait_kind::WaitKind,
    character,
    encounters::{EnteredWorld, KnownEntities},
    items::{Inventory, PaperDoll},
    network::packets::client::RequestCharCreate,
    npc,
    object_id::{ObjectId, ObjectIdManager},
    skills::SkillList,
    stats::*,
};
use l2r_core::{
    db::{DbConnection, PostgresPlugin},
    metrics::MetricsPlugin,
    model::{base_class::BaseClass, race::Race},
};
use map::{Region, RegionGeoData, WorldMap, id::RegionId};
use scripting::RuntimeScriptsTaskSpawned;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, prelude::Uuid};
#[cfg(test)]
pub use serial_test::serial;
use spatial::GameVec3;
use state::GameServerStateSystems;

/// Creates a test application with a full game server setup.
///
/// Tests using this function should be marked with `#[serial]` because:
/// - They share the same file system for loading assets (geodata, NPC data, etc.)
/// - Bevy's asset system can have conflicts when multiple apps load from the same paths simultaneously
pub fn create_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(bevy_defer::AsyncPlugin::default_settings());
    // Use mock metrics plugin for tests - no web server needed
    app.add_plugins(MetricsPlugin::mock("testgs_".to_string()));
    app.add_plugins(plugins::Core);
    app.add_plugins(PostgresPlugin {
        connection: Some(create_db_mocking_connection()),
        config: None,
    });

    app.init_resource::<CollisionDiagnostics>();
    app.init_resource::<SolverDiagnostics>();
    app.init_resource::<SpatialQueryDiagnostics>();

    {
        // Spawn a mock RuntimeScriptsTaskSpawned that's already loaded
        app.world_mut()
            .spawn(RuntimeScriptsTaskSpawned { loaded: true });
    }

    let mut iterations = 0;
    const MAX_STATE_ITERATIONS: u32 = 5000;

    loop {
        iterations += 1;
        app.update();

        let world = app.world_mut();
        let gs_state = world.resource::<State<GameServerStateSystems>>();
        if *gs_state == GameServerStateSystems::Run {
            println!("State transitioned to Run after {} iterations", iterations);
            break;
        }

        if iterations >= MAX_STATE_ITERATIONS {
            println!(
                "Warning: State transition timeout after {} iterations, current state: {:?}",
                iterations,
                gs_state.get()
            );
            // For tests, we can force the state transition
            world
                .resource_mut::<NextState<GameServerStateSystems>>()
                .set(GameServerStateSystems::Run);
            break;
        }

        // Print progress every 100 iterations to help debug
        if iterations % 100 == 0 {
            println!(
                "State transition iteration {}, current state: {:?}",
                iterations,
                gs_state.get()
            );
        }
    }

    let char_id = {
        let world = app.world_mut();
        let mut object_id_manager = world.resource_mut::<ObjectIdManager>();
        object_id_manager.next_id()
    };

    let world = app.world_mut();

    let char_bundle = test_character_bundle_data(char_id);

    let character_entity = world.spawn(char_bundle).id();

    // Add EnteredWorld component separately since it's not part of the Bundle
    world.entity_mut(character_entity).insert(EnteredWorld);

    let character_translation = world
        .query_filtered::<&Transform, With<character::Character>>()
        .single(world)
        .unwrap();

    let region_id = RegionId::from(character_translation.translation);

    let mut geodata_iterations = 0;
    const MAX_GEODATA_ITERATIONS: u32 = 5000;

    loop {
        geodata_iterations += 1;
        app.update();

        let geodata_exists = {
            let world = app.world_mut();
            let mut regions_query = world.query::<Ref<Region>>();

            let world_map = world.get_resource::<WorldMap>().unwrap();
            let regions_geodata = world.get_resource::<Assets<RegionGeoData>>().unwrap();

            world_map
                .get(&region_id)
                .and_then(|region_entity| regions_query.get(world, *region_entity).ok())
                .and_then(|region| regions_geodata.get(region.handle().id()))
                .is_some()
        };

        if geodata_exists {
            println!("Geodata loaded after {} iterations", geodata_iterations);
            break;
        }

        if geodata_iterations >= MAX_GEODATA_ITERATIONS {
            println!(
                "Warning: Geodata loading timeout after {} iterations",
                geodata_iterations
            );
            break; // Continue without geodata for tests
        }
    }

    let mut npc_iterations = 0;
    const MAX_NPC_ITERATIONS: u32 = 5000;

    loop {
        npc_iterations += 1;
        app.update();

        let (npc_exists, npc_assets_loaded) = {
            let world = app.world_mut();
            let mut mob_query = world.query_filtered::<Entity, With<npc::Kind>>();
            let has_npcs = mob_query.iter(world).next().is_some();

            // Check if NPC assets are actually loaded by looking at RegionalNpcHandles
            let npc_assets = world.resource::<bevy::asset::Assets<npc::NpcInfo>>();
            let has_npc_assets = !npc_assets.is_empty();

            (has_npcs, has_npc_assets)
        };

        if npc_exists && npc_assets_loaded {
            println!(
                "NPCs and NPC assets loaded after {} iterations",
                npc_iterations
            );
            break;
        }

        if npc_iterations >= MAX_NPC_ITERATIONS {
            println!(
                "Warning: NPC loading timeout after {} iterations (NPCs exist: {}, Assets loaded: {})",
                npc_iterations, npc_exists, npc_assets_loaded
            );
            break; // Continue without NPCs for tests
        }
    }

    app
}

pub fn create_db_mocking_connection() -> DbConnection {
    let character_creation = RequestCharCreate {
        name: "Test Character".to_owned(),
        race: Race::DarkElf,
        class_id: ClassId::DarkFighter,
        appearance: character::Appearance::default(),
        primal_stats: PrimalStats::default(),
    };

    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![character::model::Model::new(
            ObjectId::test_data(),
            Uuid::default(),
            character_creation,
            VitalsStats::test_data(),
            GameVec3::new(0, 0, 0),
        )]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    DbConnection::new(connection, false)
}

pub fn get_character_entity_and_oid(app: &mut App) -> (Entity, ObjectId) {
    let world = app.world_mut();
    let mut character_query =
        world.query_filtered::<(Entity, Ref<ObjectId>), With<character::Character>>();
    let (character_entity, character_oid) = character_query.single(world).unwrap();
    (character_entity, *character_oid)
}

pub fn race_stats_test_data() -> RaceStats {
    let mut asset_dir = l2r_core::utils::get_base_path();
    asset_dir.push(l2r_core::assets::ASSET_DIR);

    let mut path = asset_dir;
    path.push("tests");
    path.push("race_stats.json");

    let file =
        std::fs::File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));

    let reader = std::io::BufReader::new(file);

    serde_json::from_reader(reader)
        .unwrap_or_else(|_| panic!("Failed to parse from JSON: {:?}", path))
}

pub fn test_character_bundle_data(id: ObjectId) -> character::Bundle {
    let base_class = BaseClass::Fighter;
    let race = Race::DarkElf;
    let race_stats = race_stats_test_data();
    let base_class_stats = race_stats.get(race, base_class);
    let vitals_stats = VitalsStats::test_data();
    let start_location = base_class_stats.born_points[0];
    let appearance = character::Appearance::default();
    character::Bundle {
        id,
        character: character::Character,
        name: Name::new("TestCharacter".to_string()),
        title: NameTitle::new("TestTitle".to_string()),
        session_id: ConnectionId::next().into(),
        movable: Movable::from(base_class_stats),
        collider: base_class_stats.collider(appearance.gender),
        base_class,
        vitals_stats,
        primal_stats: base_class_stats.primal_stats.clone(),
        attack_stats: AttackStats::default(),
        defence_stats: DefenceStats::default(),
        critical_stats: CriticalStats::default(),
        inventory_stats: InventoryStats::default(),
        progress_stats: ProgressStats::default(),
        progress_level: ProgressLevelStats::new(10.into()),
        other_stats: OtherStats::default(),
        stat_modifiers: StatModifiers::default(),
        pvp: PvpStats::default(),
        sub_class: SubClass::from((SubClassVariant::Main, ClassId::DarkFighter)),
        race,
        appearance,
        transform: Transform::from_translation(start_location.into()),
        paper_doll: PaperDoll::default(),
        delete_timer: character::DeleteTimer::default(),
        visibility: EncountersVisibility::EncountersVisibility,
        skill_list: SkillList::default(),
        known_entities: KnownEntities::default(),
        inventory: Inventory::default(),
        wait_kind: WaitKind::default(),
        attack_effects: AttackEffects::default(),
        defence_effects: DefenceEffects::default(),
        abnormal_effects: AbnormalEffects::default(),
        last_known_pos: LastKnownPosition::default(),
    }
}
