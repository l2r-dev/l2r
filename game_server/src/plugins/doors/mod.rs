use avian3d::prelude::{CollisionEventsEnabled, Sensor};
use bevy::prelude::*;
use game_core::{
    action::target::Targetable,
    collision_layers::Layer,
    object_id::ObjectIdManager,
    stats::{DefenceStat, DefenceStats, EncountersVisibility, Stats, VitalsStat, VitalsStats},
};
use map::{DoorsComponentsPlugin, MeshInfo, Zone, ZoneKind};

mod query;

pub use query::*;

pub struct DoorsPlugin;
impl Plugin for DoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DoorsComponentsPlugin);

        app.add_observer(door_added);
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

        // Zones are Sensors by default
        // Set collision layers based on door status and collision check
        match (door.check_collision, door.status) {
            (true, map::DoorStatus::Close) => {
                // Closed door - solid environment that blocks characters
                commands.entity(entity).try_remove::<Sensor>();
                commands.entity(entity).insert(Layer::environment_solid());
            }
            (true, map::DoorStatus::Open) => {
                // Open door - passable environment, doesn't block movement
                commands
                    .entity(entity)
                    .insert(Layer::environment_passable());
            }
            (false, _) => {
                // No collision check - pure sensor/trigger
                commands.entity(entity).insert(Layer::sensor());
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
            door.status,
            MeshInfo::default(),
        ));

        // Doors do not need collision events
        commands.entity(entity).try_remove::<CollisionEventsEnabled>();
    }

    Ok(())
}
