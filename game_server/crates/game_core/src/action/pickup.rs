use crate::{attack::Attacking, movement::Following, npc::DialogRequest};
use bevy::prelude::*;
use bevy_ecs::{
    component::{ComponentHook, HookContext, Immutable, StorageType},
    world::DeferredWorld,
};
use strum::Display;

pub struct PickupComponentsPlugin;
impl Plugin for PickupComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PickupRequest>();
    }
}

#[derive(Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct PickupRequest(pub Entity);

impl Component for PickupRequest {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Immutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            world
                .commands()
                .entity(context.entity)
                .remove::<(Attacking, DialogRequest, Following)>();
        })
    }
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PickupMetric {
    ItemsPickedUp,
}
