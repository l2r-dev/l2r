use bevy::prelude::*;

pub mod model;
pub mod pickup;
pub mod target;
pub mod wait_kind;

pub struct UseActionPlugin;
impl Plugin for UseActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(target::TargetComponentsPlugin)
            .add_plugins(pickup::PickupComponentsPlugin);
    }
}
