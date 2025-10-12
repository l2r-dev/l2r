use bevy::prelude::*;

#[derive(Clone, Component, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub enum Visible {
    Hidden,
    #[default]
    Visible,
}
