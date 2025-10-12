use bevy::prelude::*;
use game_core::{
    action::{
        model::CoreAction,
        wait_kind::{Sit, WaitKind},
    },
    animation::AnimationTimer,
    encounters::EnteredWorld,
    movement::MoveTarget,
    network::{broadcast::ServerPacketBroadcast, packets::server::ChangeWaitType},
    object_id::ObjectId,
};
use std::time::Duration;

pub struct SitStandPlugin;
impl Plugin for SitStandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_action);
        app.add_systems(Update, broadcast_sit_stand);
    }
}

fn handle_action(
    action: Trigger<CoreAction>,
    mut commands: Commands,
    possible_sitters: Query<Has<Sit>>,
) {
    let entity = action.target();
    if action.event() == &CoreAction::SitStand
        && let Ok(has_sit) = possible_sitters.get(entity)
    {
        commands
            .entity(entity)
            .insert(AnimationTimer::new(Duration::from_secs(2)));

        if has_sit {
            commands
                .entity(entity)
                .remove::<Sit>()
                .try_insert(WaitKind::Stand);
        } else {
            commands
                .entity(entity)
                .try_insert(WaitKind::Sit)
                .try_insert(Sit)
                .remove::<MoveTarget>();
        }
    }
}

fn broadcast_sit_stand(
    mut commands: Commands,
    query: Query<
        (Entity, Ref<ObjectId>, Has<Sit>, Ref<Transform>),
        (Changed<WaitKind>, With<EnteredWorld>),
    >,
) {
    for (entity, object_id, has_sit, transform) in &mut query.iter() {
        let packet = ChangeWaitType::new(
            *object_id,
            if has_sit {
                WaitKind::Sit
            } else {
                WaitKind::Stand
            },
            transform.translation,
        );
        commands.trigger_targets(ServerPacketBroadcast::new(packet.into()), entity);
    }
}
