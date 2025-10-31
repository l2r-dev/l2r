use avian3d::prelude::Sensor;
use bevy::prelude::*;
use game_core::stats::{DefenceStat, DefenceStats, Stats, VitalsStat, VitalsStats};
use map::{DoorsComponentsPlugin, Zone, ZoneKind};

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
    zones: Query<Ref<Zone>>,
) -> Result<()> {
    // Handle door zone added
    let entity = added.target();
    let zone = zones.get(entity)?;
    if let ZoneKind::Door(door_kind) = zone.kind() {
        let mut vitals = VitalsStats::default();
        vitals.insert(VitalsStat::Hp, door_kind.hp_max as f32);
        vitals.insert(VitalsStat::MaxHp, door_kind.hp_max as f32);

        let mut defence_stats = DefenceStats::default();
        defence_stats.insert(DefenceStat::PDef, door_kind.p_def as f32);
        if let Some(m_def) = door_kind.m_def {
            defence_stats.insert(DefenceStat::MDef, m_def as f32);
        }

        if door_kind.check_collision {
            commands.entity(entity).try_remove::<Sensor>();
        }
    }

    Ok(())
}
