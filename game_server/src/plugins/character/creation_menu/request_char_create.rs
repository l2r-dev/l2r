use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    account::Account,
    character, items,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{GameClientPacket, RequestCharCreate},
            server::{
                CharacterCreationFailReason, CharacterCreationFailed, CharacterCreationSuccess,
                GameServerPacket,
            },
        },
        session::{GameServerSession, PacketReceiveParams},
    },
    object_id::{ObjectId, ObjectIdManager},
    stats::*,
};
use l2r_core::{
    db::{Repository, RepositoryManager, TypedRepositoryManager},
    model::session::L2rSession,
};
use rand::seq::SliceRandom;
use sea_orm::ColumnTrait;
use spatial::GameVec3;

pub(crate) struct RequestCharCreatePlugin;
impl Plugin for RequestCharCreatePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_packet);
    }
}

fn handle_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestCharCreate(ref packet) = event.packet else {
        return Ok(());
    };
    let session_entity = receive_params.session(&event.connection.id())?;

    if packet.name.len() > 16 {
        commands.trigger_targets(
            GameServerPacket::from(CharacterCreationFailed::new(
                CharacterCreationFailReason::SixteenEngChars,
            )),
            session_entity,
        );
        return Err(BevyError::from(format!(
            "Name too long: {}",
            session_entity
        )));
    }

    let chars_table = receive_params.character_table(&event.connection.id())?;

    if chars_table.is_max_len() {
        commands.trigger_targets(
            GameServerPacket::from(CharacterCreationFailed::new(
                CharacterCreationFailReason::TooManyCharacters,
            )),
            session_entity,
        );
        return Err(BevyError::from(format!(
            "Account already has maximum chars: {:?}",
            session_entity
        )));
    }

    let char_create_request = packet.clone();
    commands
        .spawn_task(move || async move { create_task(session_entity, char_create_request).await });

    Ok(())
}

async fn create_task(
    session_entity: Entity,
    char_create_request: RequestCharCreate,
) -> Result<(), AccessError> {
    let Ok(character_repository) = AsyncWorld
        .resource::<RepositoryManager>()
        .get(|manager| manager.typed::<ObjectId, character::model::Entity>())?
    else {
        return Ok(());
    };

    // Check name uniqueness
    let existing = character_repository
        .find_with_conditions(
            [character::model::Column::Name.eq(char_create_request.name.as_str())],
        )
        .await;

    let account_id = AsyncWorld
        .entity(session_entity)
        .component::<Account>()
        .get(|account| account.id())?;

    if existing.is_err() {
        log::error!(
            "Failed to check name uniqueness for account: {:?}",
            account_id
        );
        send_fail_reason_to_session(CharacterCreationFailReason::CreationFailed, session_entity)
            .await;
        return Ok(());
    }

    for model in existing.unwrap() {
        if model.name == char_create_request.name {
            log::warn!(
                "Character name already exists: {:?}",
                char_create_request.name
            );
            send_fail_reason_to_session(
                CharacterCreationFailReason::NameAlreadyExists,
                session_entity,
            )
            .await;
            return Ok(());
        }
    }

    let (vitals_stats, init_pos) = get_init_pos_and_vitals(char_create_request.clone()).await;

    let object_id = AsyncWorld
        .resource::<ObjectIdManager>()
        .get_mut(|object_id_manager| object_id_manager.next_id())?;

    log::debug!("New char vitals: {:?}", vitals_stats);

    let new_char = character::model::Model::new(
        object_id,
        account_id,
        char_create_request,
        vitals_stats,
        init_pos,
    );

    let create_result = character_repository.create(&new_char).await;

    if create_result.is_err() {
        log::error!("Failed to create character: {:?}", create_result);
        send_fail_reason_to_session(CharacterCreationFailReason::CreationFailed, session_entity)
            .await;
        return Ok(());
    }

    let character = create_result.unwrap();

    AsyncWorld.apply_command(move |world: &mut World| {
        let connection_id = {
            let session = world.entity(session_entity).get::<GameServerSession>();

            match session {
                Some(session) => session.id(),
                None => {
                    log::error!("Failed to get session for entity: {:?}", session_entity);
                    return;
                }
            }
        };

        let bundle =
            character::Bundle::new(character, [items::Id::default(); 26], connection_id, world);

        let Ok(mut session_entity_mut) = world.get_entity_mut(session_entity) else {
            log::error!("Failed to get mutable session entity: {:?}", session_entity);
            return;
        };

        let char_table = session_entity_mut.get_mut::<character::Table>();

        let Some(mut table) = char_table else {
            log::error!(
                "Failed to get character table for session: {:?}",
                session_entity
            );
            return;
        };

        if let Err(e) = table.add_bundle(bundle, [items::Id::default(); 26]) {
            log::error!("Failed to add character to table: {:?}", e);
            world.trigger_targets(
                GameServerPacket::from(CharacterCreationFailed::new(
                    CharacterCreationFailReason::default(),
                )),
                session_entity,
            );
            return;
        }

        world.trigger_targets(
            GameServerPacket::from(CharacterCreationSuccess),
            session_entity,
        );
    });

    Ok(())
}

async fn get_init_pos_and_vitals(
    char_create_request: RequestCharCreate,
) -> (VitalsStats, GameVec3) {
    AsyncWorld.run(|world: &mut World| {
        let stats_table = world.resource::<StatsTable>();

        let progress_level = ProgressLevelStats::new(1.into());
        let stat_modifiers = StatModifiers::default();

        let stat_formula_registry = world.resource::<StatFormulaRegistry>();
        let class_tree = stats_table.class_tree_world(world);
        let race_stats = stats_table.race_stats_world(world);

        let base_class = class_tree.get_base_class(char_create_request.class_id);
        let base_class_stats = race_stats.get(char_create_request.race, base_class);

        let base_vitals = stats_table.vitals_stats_world(
            world,
            char_create_request.class_id,
            progress_level.level(),
        );

        let base_vitals = match base_vitals {
            Some(v) => v,
            None => {
                log::error!(
                    "Failed to get vitals stats for class: {:?}",
                    char_create_request.class_id
                );
                return (VitalsStats::default(), GameVec3::default());
            }
        };

        let mut vitals_stats = base_vitals.clone();

        let params = StatsCalculateParams::new(
            stat_formula_registry,
            &base_class_stats.primal_stats,
            &progress_level,
            base_class,
            None,
            &stat_modifiers,
            true,
            false,
        );

        vitals_stats.calculate(params, Some(base_vitals.current()));
        vitals_stats.fill_current_from_max();

        let init_pos = *base_class_stats
            .born_points
            .choose(&mut rand::thread_rng())
            .unwrap();

        (vitals_stats, init_pos)
    })
}

async fn send_fail_reason_to_session(
    fail_reason: CharacterCreationFailReason,
    session_entity: Entity,
) {
    AsyncWorld.apply_command(move |world: &mut World| {
        world.trigger_targets(
            GameServerPacket::from(CharacterCreationFailed::new(fail_reason)),
            session_entity,
        );
    });
}
