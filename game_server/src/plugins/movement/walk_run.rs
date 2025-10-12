use bevy::prelude::*;
use game_core::{
    action::model::CoreAction,
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{ChangeMoveType, UserInfoUpdated},
    },
    object_id::ObjectId,
    stats::{Movable, MovementStat},
};

pub struct WalkRunPlugin;
impl Plugin for WalkRunPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_action);
    }
}

fn handle_action(
    action: Trigger<CoreAction>,
    mut commands: Commands,
    mut movables: Query<(Ref<ObjectId>, Mut<Movable>)>,
    mut user_info_updates: EventWriter<UserInfoUpdated>,
) {
    let entity = action.target();
    if action.event() == &CoreAction::WalkRun
        && let Ok((object_id, mut movable)) = movables.get_mut(entity)
    {
        match movable.move_type() {
            MovementStat::Walk => {
                movable.set_move_type(MovementStat::Run);
            }
            MovementStat::Run => {
                movable.set_move_type(MovementStat::Walk);
            }
            _ => {}
        }
        user_info_updates.write(UserInfoUpdated(entity));
        commands.trigger_targets(
            ServerPacketBroadcast::new(ChangeMoveType::new(*object_id, movable.move_type()).into()),
            entity,
        );
    }
}
