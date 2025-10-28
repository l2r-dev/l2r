use bevy::prelude::*;
use strum::Display;

pub struct PickupComponentsPlugin;
impl Plugin for PickupComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PickupRequest>();
    }
}

#[derive(Clone, Component, Copy, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PickupRequest(pub Entity);

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PickupMetric {
    ItemsPickedUp,
}
