use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    chat,
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::{BypassCommand, BypassCommandExecuted, GameClientPacket},
            server::{CreatureSay, GameServerPacket},
        },
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
};

pub(crate) struct BypassCommandPlugin;
impl Plugin for BypassCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle).add_observer(handle_unknown);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
) {
    let event = receive.event();
    if let GameClientPacket::BypassCommand(ref packet) = event.packet
        && let Ok(entity) = receive_params.character(&event.connection.id())
    {
        commands.trigger_targets(BypassCommandExecuted::from(packet.clone()), entity);
    }
}

fn handle_unknown(
    trigger: Trigger<BypassCommandExecuted>,
    mut commands: Commands,
    object_ids: Query<Ref<ObjectId>>,
) {
    let BypassCommandExecuted(cmd) = trigger.event();
    let entity = trigger.target();
    if let BypassCommand::Unknown(unknown_command) = cmd {
        let Ok(object_id) = object_ids.get(entity) else {
            return;
        };
        commands.trigger_targets(
            GameServerPacket::from(CreatureSay::new(
                *object_id,
                "System".to_string(),
                vec![format!(
                    "Unknown command: {}. Please check the command syntax.",
                    unknown_command
                )],
                chat::Kind::General,
                None,
            )),
            entity,
        );
    }
}
