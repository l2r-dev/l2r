use bevy::{log, prelude::*};
use bevy_defer::AsyncCommandsExtension;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
    shortcut,
    stats::SubClass,
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};

pub(crate) struct RequestShortcutDeletePlugin;

impl Plugin for RequestShortcutDeletePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    repo_manager: Res<RepositoryManager>,
    characters: Query<(Ref<ObjectId>, Ref<SubClass>)>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestShortcutDelete(ref packet) = event.packet else {
        return Ok(());
    };

    let entity = receive_params.character(&event.connection.id())?;
    let (char_id, sub_class) = characters.get(entity)?;
    let sub_class = *sub_class;
    let char_id = *char_id;

    let shortcut_repository =
        repo_manager.typed::<shortcut::model::ShortcutPK, shortcut::model::Entity>()?;
    let slot_id = packet.0;

    commands.spawn_task(move || async move {
        let result = shortcut_repository
            .delete_by_id(shortcut::model::ShortcutPK {
                char_id,
                slot_id,
                class_variant: sub_class.into(),
            })
            .await;

        if let Err(err) = result {
            log::error!(
                "Character: {}, Failed to delete shortcut: {:?}",
                char_id,
                err
            );
            return Err(err.into());
        }

        Ok(())
    });
    Ok(())
}
