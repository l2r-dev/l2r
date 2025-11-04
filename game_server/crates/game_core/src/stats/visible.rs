use bevy::prelude::*;

#[repr(u8)]
#[derive(Clone, Component, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub enum EncountersVisibility {
    Hidden,
    #[default]
    Visible,
}

impl From<bool> for EncountersVisibility {
    fn from(hidden: bool) -> Self {
        if hidden {
            EncountersVisibility::Hidden
        } else {
            EncountersVisibility::Visible
        }
    }
}
