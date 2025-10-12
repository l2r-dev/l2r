use bevy::prelude::*;
use game_core::{
    action::model::CoreAction,
    network::{
        broadcast::ServerPacketBroadcast,
        packets::server::{Social, SocialAction},
    },
    object_id::ObjectId,
};

pub struct SeeDebugPlugin;
impl Plugin for SeeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_action);
    }
}

fn handle_action(
    action: Trigger<CoreAction>,
    mut commands: Commands,
    object_ids: Query<Ref<ObjectId>>,
) -> Result<()> {
    let entity = action.target();
    let object_id = *object_ids.get(entity)?;

    if let Ok(social) = Social::try_from(*action.event()) {
        let action = SocialAction::new(object_id, social);
        commands.trigger_targets(ServerPacketBroadcast::new(action.into()), entity);
    }

    Ok(())
}
