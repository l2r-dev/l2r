use avian3d::prelude::CollisionEventsEnabled;
use bevy::prelude::*;
use game_core::{
    action::target::{SelectedTarget, Targetable},
    attack::AttackingList,
    network::{
        broadcast::{BroadcastScope, ServerPacketBroadcast},
        packets::server::{
            BroadcastDoorStatusUpdate, DoorStatusUpdate, GameServerPacket, StaticObjectInfo,
        },
    },
    object_id::ObjectIdManager,
    stats::{
        DefenceEffects, DefenceStat, DefenceStats, EncountersVisibility, Stats, VitalsStat,
        VitalsStats,
    },
};
use map::{DoorCommand, DoorStatus, DoorsComponentsPlugin, MeshInfo, Zone, ZoneKind};
use physics::GameLayer;

mod query;

pub use query::*;

const DOOR_BROADCAST_RADIUS: f32 = 3000.0;

pub struct DoorsPlugin;
impl Plugin for DoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DoorsComponentsPlugin);

        app.add_observer(door_added)
            .add_observer(broadcast_door_status_update)
            .add_observer(changed_door_status)
            .add_observer(handle_door_command);
    }
}

fn door_added(
    added: Trigger<OnAdd, Zone>,
    mut commands: Commands,
    mut object_id_manager: ResMut<ObjectIdManager>,
    zones: Query<Ref<Zone>>,
) -> Result<()> {
    let entity = added.target();
    let zone = zones.get(entity)?;
    if let ZoneKind::Door(door) = zone.kind() {
        let mut vitals = VitalsStats::default();
        vitals.insert(VitalsStat::Hp, door.max_hp as f32);
        vitals.insert(VitalsStat::MaxHp, door.max_hp as f32);

        let mut defence_stats = DefenceStats::default();
        defence_stats.insert(DefenceStat::PDef, door.p_def as f32);
        if let Some(m_def) = door.m_def {
            defence_stats.insert(DefenceStat::MDef, m_def as f32);
        }

        // Set collision layers based on door status and collision check
        match (door.check_collision, door.status) {
            (true, map::DoorStatus::Close) => {
                // Closed door - solid environment that blocks characters (Environment layer)
                commands
                    .entity(entity)
                    .insert(GameLayer::environment_solid());
            }
            (true, map::DoorStatus::Open) => {
                // Open door - passable environment that doesn't block movement (EnvironmentPassable layer)
                commands
                    .entity(entity)
                    .insert(GameLayer::environment_passable());
            }
            (false, _) => {
                commands
                    .entity(entity)
                    .insert(GameLayer::environment_passable());
            }
        }

        if door.targetable {
            commands.entity(entity).insert(Targetable);
        }

        let visibility: EncountersVisibility = door.hidden.into();

        let object_id = object_id_manager.next_id();
        object_id_manager.register_entity(entity, object_id);

        commands.entity(entity).insert((
            object_id,
            visibility,
            vitals,
            defence_stats,
            DefenceEffects::default(),
            door.status,
            MeshInfo::default(),
            AttackingList::default(),
        ));

        // Doors do not need collision events
        commands
            .entity(entity)
            .try_remove::<CollisionEventsEnabled>();
    }

    Ok(())
}

fn handle_door_command(
    trigger: Trigger<DoorCommand>,
    mut commands: Commands,
    player_query: Query<Ref<SelectedTarget>>,
    door_query: Query<DoorQuery>,
) -> Result<()> {
    let player_entity = trigger.target();
    let door_command = *trigger.event();

    let selected_target = player_query.get(player_entity)?;
    let door_entity = selected_target.0;

    let door = door_query.get(door_entity)?;

    let map::ZoneKind::Door(door_kind) = door.zone.kind() else {
        return Ok(());
    };

    let new_status = DoorStatus::from(door_command);
    if *door.status == new_status {
        return Ok(());
    }

    if door_kind.check_collision {
        match new_status {
            DoorStatus::Close => {
                commands
                    .entity(door_entity)
                    .insert(GameLayer::environment_solid());
            }
            DoorStatus::Open => {
                commands
                    .entity(door_entity)
                    .insert(GameLayer::environment_passable());
            }
        }
    }
    commands.entity(door_entity).insert(new_status);
    Ok(())
}

fn changed_door_status(
    changed: Trigger<OnReplace, DoorStatus>,
    mut commands: Commands,
) -> Result<()> {
    let door_entity = changed.target();
    commands.trigger_targets(BroadcastDoorStatusUpdate, door_entity);
    Ok(())
}

fn broadcast_door_status_update(
    broadcast: Trigger<BroadcastDoorStatusUpdate>,
    door_query: Query<DoorQuery>,
    mut commands: Commands,
) -> Result<()> {
    let door_entity = broadcast.target();
    let door = door_query.get(door_entity)?;
    let map::ZoneKind::Door(door_kind) = door.zone.kind() else {
        return Ok(());
    };

    let is_enemy = true; // TODO: Check if door is_enemy, now just consider all doors as enemy
    commands.trigger_targets(
        ServerPacketBroadcast {
            packet: GameServerPacket::from(DoorStatusUpdate::new(
                door_kind,
                *door.object_id,
                door.vitals.as_ref(),
                *door.status,
                is_enemy,
            )),
            scope: BroadcastScope::Radius(DOOR_BROADCAST_RADIUS),
        },
        door_entity,
    );

    commands.trigger_targets(
        ServerPacketBroadcast {
            packet: GameServerPacket::from(StaticObjectInfo::door(
                *door.object_id,
                door_kind,
                door.vitals.as_ref(),
                *door.status,
                *door.mesh_info,
                is_enemy,
            )),
            scope: BroadcastScope::Radius(DOOR_BROADCAST_RADIUS),
        },
        door_entity,
    );

    Ok(())
}
