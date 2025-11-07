use bevy::{log, prelude::*};
use bevy_defer::{AccessError, AsyncAccess, AsyncWorld};
use game_core::{
    network::packets::server::{GameServerPacket, ShortcutInit},
    object_id::ObjectId,
    shortcut::{self, ShortcutComponentsPlugin},
    stats::SubClass,
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::ColumnTrait;

mod shortcut_delete;
mod shortcut_registration;

/// The plugin for handling shortcuts in the game to use it with hotkeys or mouse clicks.
/// Player can register shortcuts for items, skills, actions, macros, recipes etc.
/// uses [`ShortcutRegistered`], [`ShortcutInit`] server packets
/// alongside with clients: [`RequestShortcutRegistration`], [`RequestShortcutDelete`]
pub(crate) struct ShortcutPlugin;

impl Plugin for ShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShortcutComponentsPlugin)
            .add_plugins(shortcut_registration::RequestShortcutRegistrationPlugin)
            .add_plugins(shortcut_delete::RequestShortcutDeletePlugin);
    }
}

pub async fn shortcut_init_task(char_entity: Entity) -> Result<(), AccessError> {
    let Ok(shortcut_repository) = AsyncWorld
        .resource::<RepositoryManager>()
        .get(|registry| registry.typed::<shortcut::model::ShortcutPK, shortcut::model::Entity>())?
    else {
        return Ok(());
    };

    let char_id = AsyncWorld
        .entity(char_entity)
        .component::<ObjectId>()
        .get(|component| *component)?;

    let sub_class = AsyncWorld
        .entity(char_entity)
        .component::<SubClass>()
        .get(|component| *component)?;

    let result = shortcut_repository
        .find_with_conditions([
            shortcut::model::Column::CharId.eq(char_id),
            shortcut::model::Column::ClassVariant.eq(sub_class.variant()),
        ])
        .await;

    if let Err(err) = result {
        log::error!("Character: {}, Error loading shortcuts: {:?}", char_id, err);
        return Err(err.into());
    }

    let shortcuts = result
        .unwrap_or_default()
        .into_iter()
        .map(shortcut::Shortcut::from)
        .collect::<Vec<_>>();

    let shortcut_init = ShortcutInit::from(shortcuts);

    AsyncWorld.apply_command(move |world: &mut World| {
        world.trigger_targets(GameServerPacket::from(shortcut_init), char_entity);
    });

    Ok(())
}
