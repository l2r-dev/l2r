use bevy::prelude::*;
use bevy_defer::{
    AccessError, AppReactorExtension, AsyncAccess, AsyncCommandsExtension, AsyncExtension,
    AsyncWorld,
};
use game_core::{
    items::{self, InventoryComponentsPlugin, InventoryLoad, SpawnExisting, model},
    object_id::ObjectId,
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::ColumnTrait;

mod added;
mod destroy;
mod drop;
mod equip;
mod unequip;

use added::*;
pub use destroy::*;
pub use drop::*;
pub use equip::*;
pub use unequip::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InventoryComponentsPlugin);

        app.add_observer(add_in_inventory)
            .add_observer(add_non_stackable)
            .add_observer(add_stackable)
            .add_observer(destroy_item);

        app.add_plugins(DropItemPlugin)
            .add_plugins(EquipItemPlugin)
            .add_plugins(UnequipItemPlugin);

        app.spawn_task(async { load_inventory_from_db().await })
            .react_to_event::<InventoryLoad>();
    }
}
async fn load_inventory_from_db() -> Result<(), AccessError> {
    while let Ok(load_event) = AsyncWorld.get_next_event::<InventoryLoad>().await {
        let entity = load_event.0;

        let char_id = AsyncWorld
            .entity(entity)
            .component::<ObjectId>()
            .get(|id| *id)?;

        let Ok(items_repository) = AsyncWorld
            .resource::<RepositoryManager>()
            .get(|manager| manager.typed::<ObjectId, items::model::Entity>())?
        else {
            return Ok(());
        };

        let item_models = items_repository
            .find_with_conditions([model::Column::OwnerId.eq(char_id)])
            .await?;

        AsyncWorld.apply_command(move |world: &mut World| {
            world.trigger_targets(
                SpawnExisting {
                    item_models,
                    dropped_entity: None,
                    silent: true, // Don't show system messages during initial inventory loading
                },
                entity,
            );
            world.commands().spawn_task(move || async move {
                crate::plugins::shortcuts::shortcut_init_task(entity).await
            });
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests;
