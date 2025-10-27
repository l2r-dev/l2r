use crate::{
    attack::Attacking,
    movement::{Following, MoveTarget, MoveToEntity},
};
use bevy::prelude::*;
use bevy_enum_tag::EnumComponentTag;
use num_enum::IntoPrimitive;
use strum::Display;

#[derive(
    EnumComponentTag,
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    IntoPrimitive,
)]
#[repr(u32)]
pub enum WaitKind {
    Sit,
    #[default]
    Stand,
    FakeDeath,
    UnfakeDeath,
}

// Expose EnumComponentTag generated methods
pub use wait_kind::*;

pub fn sit_added(trigger: Trigger<OnAdd, Sit>, mut commands: Commands) {
    let entity = trigger.target();

    commands
        .entity(entity)
        .remove::<MoveTarget>()
        .remove::<MoveToEntity>()
        .remove::<Following>()
        .remove::<Attacking>();
}
