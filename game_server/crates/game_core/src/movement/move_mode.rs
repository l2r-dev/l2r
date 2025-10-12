use bevy::prelude::*;
use num_enum::TryFromPrimitive;
use strum::Display;

#[derive(Clone, Copy, Debug, Default, Display, Eq, Hash, PartialEq, Reflect, TryFromPrimitive)]
#[repr(u32)]
pub enum MoveMode {
    #[default]
    MoveTarget,
    Keyboard,
}
