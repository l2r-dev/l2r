use bevy::{log, prelude::*};
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, ShortcutRegistered},
        },
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
    shortcut,
    stats::SubClass,
};
use l2r_core::db::{
    PrimaryKeyColumns, Repository, RepositoryManager, TypedRepositoryManager, UpdatableModel,
};
use sea_orm::sea_query::OnConflict;

pub(crate) struct RequestShortcutRegistrationPlugin;

impl Plugin for RequestShortcutRegistrationPlugin {
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
    let GameClientPacket::RequestShortcutRegistration(ref packet) = event.packet else {
        return Ok(());
    };

    let entity = receive_params.character(&event.connection.id())?;
    let (char_id, sub_class) = characters.get(entity)?;
    let sub_class = *sub_class;
    let char_id = *char_id;

    let shortcut = shortcut::Shortcut::from_packet(sub_class.into(), packet);
    let shortcut_repository =
        repo_manager.typed::<shortcut::model::ShortcutPK, shortcut::model::Entity>()?;
    let shotcut_registered = ShortcutRegistered::from(shortcut);

    commands.spawn_task(move || async move {
        let shortcut_model = shortcut.into_model(char_id);
        let result = shortcut_repository
            .create_or_update(
                &shortcut_model,
                OnConflict::columns(shortcut::model::Model::pk_columns().to_vec())
                    .update_columns(shortcut::model::Model::update_columns().to_vec())
                    .to_owned(),
            )
            .await;
        if let Err(err) = result {
            log::error!(
                "Character: {}, Error creating or updating shortcut: {:?}",
                char_id,
                err
            );
            return Err(err.into());
        }
        AsyncWorld.apply_command(move |world: &mut World| {
            world.trigger_targets(GameServerPacket::from(shotcut_registered), entity);
        });
        Ok(())
    });
    Ok(())
}
