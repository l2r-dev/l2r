mod appearing;

use avian3d::{prelude::*, spatial_query::SpatialQuery};
use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::{ParallelCommands, SystemParam},
    },
    prelude::*,
};
use game_core::{
    character::{self},
    encounters::*,
    items::{Item, ItemsDataQuery},
    movement::{MoveTarget, MoveToEntity},
    network::packets::server::{
        CharInfo, DeleteObject, GameServerPacket, MoveToLocation, MoveToPawn, NpcInfo, SpawnItem,
        StatusUpdate, StatusUpdateKind,
    },
    npc::{self, RegionalNpcInfoQuery},
    object_id::ObjectId,
    stats::*,
    teleport::TeleportInProgress,
};
use state::GameServerStateSystems;

pub struct EncountersPlugin;
impl Plugin for EncountersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EncountersComponentsPlugin)
            .add_plugins(appearing::AppearingPlugin);

        app.add_observer(handle_known_removed)
            .add_observer(handle_known_added)
            .add_observer(cleanup_known_entities);

        app.add_systems(
            Update,
            (set_relations, unset_relations)
                .chain()
                .in_set(GameServerStateSystems::Run),
        );
    }
}

const RANGE_THRESHOLD: f32 = 3000.0;

/// Entities that want to know about other entities
#[derive(QueryData)]
#[query_data(mutable)]
struct WantToKnowQuery<'a> {
    entity: Entity,
    transform: Ref<'a, Transform>,
    known_entities: Mut<'a, KnownEntities>,
}

#[derive(QueryFilter)]
struct KnowersFilter {
    not_teleporting: Without<TeleportInProgress>,
    entered_world: With<EnteredWorld>,
}

/// Entities that can be known by others
#[derive(QueryData)]
struct CanBeKnownQuery<'a> {
    entity: Entity,
    visible: Ref<'a, EncountersVisibility>,
}

// System to add entities to KnownEntities when they come into range
fn set_relations(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut want_to_knowers: Query<WantToKnowQuery, KnowersFilter>,
    can_be_known: Query<CanBeKnownQuery>,
    spatial_query: SpatialQuery,
    par_commands: ParallelCommands,
) {
    if time.elapsed_secs() - *last_time >= 0.1 {
        *last_time = time.elapsed_secs();
        let query_sphere = Collider::sphere(RANGE_THRESHOLD);

        want_to_knowers.par_iter_mut().for_each(|query_item| {
            let entity_a = query_item.entity;
            let pos_a = query_item.transform;
            let mut known_entities = query_item.known_entities;

            let filter = SpatialQueryFilter::default().with_excluded_entities([entity_a]);
            let nearby_entities = spatial_query.shape_intersections(
                &query_sphere,
                pos_a.translation,
                Quat::IDENTITY,
                &filter,
            );

            // Add visible entities that are not already known
            for entity_b in nearby_entities {
                if let Ok(known_query) = can_be_known.get(entity_b)
                    && *known_query.visible == EncountersVisibility::Visible
                    && known_entities.insert(entity_b)
                {
                    par_commands.command_scope(|mut commands| {
                        commands.trigger_targets(KnownAdded::new(entity_b), entity_a);
                    });
                }
            }
        });
    }
}

// System to remove entities from KnownEntities when conditions are no longer met
fn unset_relations(
    time: Res<Time>,
    mut last_time: Local<f32>,
    mut known: Query<WantToKnowQuery, KnowersFilter>,
    can_be_known: Query<CanBeKnownQuery>,
    spatial_query: SpatialQuery,
    object_ids: Query<Ref<ObjectId>>,
    par_commands: ParallelCommands,
) {
    if time.elapsed_secs() - *last_time >= 0.1 {
        *last_time = time.elapsed_secs();

        let query_sphere = Collider::sphere(RANGE_THRESHOLD);
        known.par_iter_mut().for_each(|query_item| {
            let entity_a = query_item.entity;
            let pos_a = query_item.transform;
            let mut known_entities = query_item.known_entities;

            let filter = SpatialQueryFilter::default().with_excluded_entities([entity_a]);
            let nearby_entities: std::collections::HashSet<Entity> = spatial_query
                .shape_intersections(&query_sphere, pos_a.translation, Quat::IDENTITY, &filter)
                .into_iter()
                .collect();

            let mut to_remove = Vec::with_capacity(known_entities.len() / 2);
            for &entity_b in known_entities.iter() {
                let entity_b_oid = match object_ids.get(entity_b) {
                    Ok(object_id) => *object_id,
                    Err(_) => {
                        to_remove.push((entity_b, ObjectId::default()));
                        continue;
                    }
                };

                // Check if entity became Hidden
                let is_hidden = can_be_known
                    .get(entity_b)
                    .map(|q| *q.visible == EncountersVisibility::Hidden)
                    .unwrap_or(false);

                // Remove if entity is no longer in range or became Hidden
                if !nearby_entities.contains(&entity_b) || is_hidden {
                    to_remove.push((entity_b, entity_b_oid));
                }
            }

            for (entity_b, entity_b_oid) in to_remove {
                known_entities.remove(&entity_b);
                par_commands.command_scope(|mut commands| {
                    commands.trigger_targets(KnownRemoved::new(entity_b_oid), entity_a);
                });
            }
        });
    }
}

#[derive(QueryData)]
struct ItemQuery<'a> {
    object_id: Ref<'a, ObjectId>,
    item: Ref<'a, Item>,
    transform: Ref<'a, Transform>,
}

#[derive(QueryData)]
struct MoveTargetQuery<'a> {
    object_id: Ref<'a, ObjectId>,
    move_target: Ref<'a, MoveTarget>,
}

#[derive(QueryData)]
struct MoveToEntityQuery<'a> {
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
    move_to_entity: Ref<'a, MoveToEntity>,
}

#[derive(QueryData)]
struct ObjectTransformQuery<'a> {
    object_id: Ref<'a, ObjectId>,
    transform: Ref<'a, Transform>,
}

#[derive(SystemParam)]
struct KnownAddedParams<'w, 's> {
    chars: Query<'w, 's, character::Query<'static>>,
    npcs: Query<'w, 's, npc::NpcQuery<'static>>,
    npc_info: RegionalNpcInfoQuery<'w, 's>,
    items: Query<'w, 's, ItemQuery<'static>, With<Collider>>,
    move_targets: Query<'w, 's, MoveTargetQuery<'static>>,
    move_to_entity: Query<'w, 's, MoveToEntityQuery<'static>>,
    object_transforms: Query<'w, 's, ObjectTransformQuery<'static>>,
    items_data_query: ItemsDataQuery<'w>,
    stats_table: StatsTableQuery<'w>,
}

fn handle_known_added(
    trigger: Trigger<KnownAdded>,
    params: KnownAddedParams,
    mut commands: Commands,
) -> Result<()> {
    let knower = trigger.target();
    let known = trigger.event().entity();

    if let Ok(character) = params.chars.get(known) {
        let base_class = params
            .stats_table
            .class_tree()
            .get_base_class(character.sub_class.class_id());
        let base_class_stats = params
            .stats_table
            .race_stats()
            .get(*character.race, base_class);

        commands.trigger_targets(
            GameServerPacket::from(CharInfo::new(
                &character,
                base_class_stats.base_speed.clone(),
            )),
            knower,
        );

        let mut status_update = StatusUpdate::new(*character.object_id);

        status_update.add(
            StatusUpdateKind::CurHp,
            character.vitals_stats.get(VitalsStat::Hp) as u32,
        );

        status_update.add(
            StatusUpdateKind::MaxHp,
            character.vitals_stats.get(VitalsStat::MaxHp) as u32,
        );

        status_update.add(
            StatusUpdateKind::CurCp,
            character.vitals_stats.get(VitalsStat::Cp) as u32,
        );

        status_update.add(
            StatusUpdateKind::MaxCp,
            character.vitals_stats.get(VitalsStat::MaxCp) as u32,
        );

        commands.trigger_targets(GameServerPacket::from(status_update), knower);
    }

    if let Ok(npc) = params.npcs.get(known) {
        let npc_model = params.npc_info.get(npc.entity)?;
        commands.trigger_targets(
            GameServerPacket::from(NpcInfo::new(&npc, npc_model.stats.speed.clone())),
            knower,
        );

        let mut status_update = StatusUpdate::new(*npc.object_id);
        status_update.add(
            StatusUpdateKind::CurHp,
            npc.condition.get(VitalsStat::Hp) as u32,
        );
        status_update.add(
            StatusUpdateKind::MaxHp,
            npc.condition.get(VitalsStat::MaxHp) as u32,
        );
        commands.trigger_targets(GameServerPacket::from(status_update), knower);
    }

    if let Ok(item_query) = params.items.get(known) {
        let item_info = params
            .items_data_query
            .get_item_info(item_query.item.id())?;

        commands.trigger_targets(
            GameServerPacket::from(SpawnItem::new(
                *item_query.object_id,
                item_query.item.id(),
                item_query.transform.translation,
                item_info.stackable(),
                item_query.item.count(),
            )),
            knower,
        );
    }

    // New known may moving right now
    if let Ok(move_target_query) = params.move_targets.get(known)
        && let Some(wp) = move_target_query.move_target.front()
    {
        commands.trigger_targets(
            GameServerPacket::from(MoveToLocation::new(
                *move_target_query.object_id,
                *wp.origin(),
                *wp.target(),
            )),
            knower,
        );
    }

    // New known may moving to another entity right now
    if let Ok(move_to_query) = params.move_to_entity.get(known)
        && let Ok(target_query) = params
            .object_transforms
            .get(move_to_query.move_to_entity.target)
    {
        commands.trigger_targets(
            GameServerPacket::from(MoveToPawn::new(
                *move_to_query.object_id,
                *target_query.object_id,
                move_to_query.transform.translation,
                target_query.transform.translation,
                move_to_query.move_to_entity.range as u32,
            )),
            knower,
        );
    }

    Ok(())
}

fn handle_known_removed(trigger: Trigger<KnownRemoved>, mut commands: Commands) {
    let event = trigger.event();
    let knower = trigger.target();

    commands.trigger_targets(
        GameServerPacket::from(DeleteObject::new(event.object_id())),
        knower,
    );
}

fn cleanup_known_entities(
    trigger: Trigger<KnownEntitiesRemoved>,
    mut knowers: Query<(Entity, Mut<KnownEntities>)>,
    mut commands: Commands,
) {
    let known_entity = trigger.target();
    let known_oid = trigger.event().object_id();
    for (knower, mut known_entities) in knowers.iter_mut() {
        known_entities.remove(&known_entity);
        commands.trigger_targets(KnownRemoved::new(known_oid), knower)
    }
}
