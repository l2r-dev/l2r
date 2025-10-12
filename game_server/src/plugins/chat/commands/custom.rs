use bevy::{log, prelude::*};
use game_core::{
    chat::{self, CustomCommandExecuted},
    network::{
        packets::server::{CreatureSay, GameServerPacket},
        session::GameServerSession,
    },
    object_id::ObjectId,
};
use std::time::Duration;

pub struct CustomCommandsPlugin;
impl Plugin for CustomCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CustomCommandExecuted>()
            .add_observer(custom_command_executed);
    }
}

fn custom_command_executed(
    custom_command: Trigger<CustomCommandExecuted>,
    mut commands: Commands,
    sessions: Query<Ref<GameServerSession>>,
    object_ids: Query<Ref<ObjectId>>,
) {
    let CustomCommandExecuted(cmd) = custom_command.event();
    let entity = custom_command.target();

    if cmd.starts_with(".ping")
        && let Ok(session) = sessions.get(entity)
    {
        let ping = session.current_ping().unwrap_or_else(|| {
            log::warn!("Ping not found for entity: {:?}", entity);
            Duration::from_secs(0)
        });
        // send it back to the client as chat
        let ping = ping.as_millis();
        let msg = format!("Ping: {ping}ms");

        let Some(object_id) = object_ids.get(entity).ok() else {
            log::warn!("ObjectId not found for entity: {:?}", entity);
            return;
        };

        let sending_packet = CreatureSay::new(
            *object_id,
            "System".to_string(),
            vec![msg],
            chat::Kind::Announcement,
            None,
        );
        commands.trigger_targets(GameServerPacket::from(sending_packet), entity);
    }
}
