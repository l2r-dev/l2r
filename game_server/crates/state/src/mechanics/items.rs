use super::GameMechanicsSystems;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use strum::EnumIter;

pub struct ItemMechanicsPlugin;

impl Plugin for ItemMechanicsPlugin {
    fn build(&self, app: &mut App) {
        configure_item_mechanics_sets(app, Update);
        configure_item_mechanics_sets(app, FixedUpdate);
    }
}

fn configure_item_mechanics_sets(app: &mut App, schedule: impl ScheduleLabel) {
    app.configure_sets(
        schedule,
        (
            ItemMechanicsSystems::Add.in_set(GameMechanicsSystems::Items),
            ItemMechanicsSystems::Equip
                .after(ItemMechanicsSystems::Add)
                .in_set(GameMechanicsSystems::Items),
            ItemMechanicsSystems::Unequip
                .after(ItemMechanicsSystems::Equip)
                .in_set(GameMechanicsSystems::Items),
            ItemMechanicsSystems::Drop
                .after(ItemMechanicsSystems::Unequip)
                .in_set(GameMechanicsSystems::Items),
            ItemMechanicsSystems::Destroy
                .after(ItemMechanicsSystems::Drop)
                .in_set(GameMechanicsSystems::Items),
        ),
    );
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq, SystemSet)]
pub enum ItemMechanicsSystems {
    Add,
    Equip,
    Unequip,
    Drop,
    Destroy,
}
