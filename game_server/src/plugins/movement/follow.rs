use super::Movement;
use bevy::prelude::*;
use game_core::{
    action::pickup::PickupRequest,
    attack::Attacking,
    movement::{FollowComponentsPlugin, FollowRequest, Following},
    network::packets::server::{ActionFail, GameServerPacket},
    path_finding::{DirectMoveRequest, InActionPathfindingTimer},
};
use map::WorldMapQuery;
use state::GameServerStateSystems;

pub struct FollowPlugin;
impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FollowComponentsPlugin);

        app.add_systems(
            FixedUpdate,
            following_changed.in_set(GameServerStateSystems::Run),
        );

        app.add_observer(follow_request_handler);
    }
}

const FOLLOW_RANGE: f32 = 100.0;

fn following_changed(
    followers: Query<
        (
            Entity,
            Ref<Following>,
            Ref<Transform>,
            Option<Ref<Movement>>,
        ),
        Without<InActionPathfindingTimer>,
    >, // Only process entities that are not on cooldown
    targets: Query<Ref<Transform>>,
    map_query: WorldMapQuery,
    mut commands: Commands,
) -> Result<()> {
    for (follower, following, follower_transform, movement) in followers.iter() {
        let target_entity = **following;
        let target_transform = targets.get(target_entity)?;

        // Check if already moving to the correct target
        if let Some(mov) = movement
            && mov.is_to_entity()
            && mov.target() == Some(target_entity)
        {
            continue;
        }

        let follower_pos = follower_transform.translation;
        let target_pos = target_transform.translation;

        // Check distance - if too close, don't need to follow
        let distance = follower_pos.distance(target_pos);
        if distance <= FOLLOW_RANGE {
            continue;
        }

        let can_move_to = map_query.can_move_to(follower_pos, target_pos);
        if can_move_to {
            commands
                .entity(follower)
                .try_insert(Movement::to_entity(target_entity, FOLLOW_RANGE));
        } else {
            commands
                .entity(follower)
                .try_insert(InActionPathfindingTimer::default());

            commands.trigger_targets(
                DirectMoveRequest {
                    entity: follower,
                    start: follower_pos,
                    target: target_pos,
                },
                follower,
            );
        }
    }
    Ok(())
}

fn follow_request_handler(
    trigger: Trigger<FollowRequest>,
    mut commands: Commands,
    followings: Query<Ref<Following>>,
) {
    let target_entity = **trigger.event();
    let follower = trigger.target();
    // If already following, send ActionFail
    if let Ok(following) = followings.get(follower)
        && **following == target_entity
    {
        commands.trigger_targets(GameServerPacket::from(ActionFail), follower);
        return;
    }
    commands
        .entity(follower)
        .remove::<(Movement, Attacking, PickupRequest)>();
    commands.entity(follower).insert(Following(target_entity));
}
