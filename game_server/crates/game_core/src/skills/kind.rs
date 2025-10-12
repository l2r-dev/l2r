use bevy::prelude::*;
use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, PartialEq, Reflect)]
#[strum(serialize_all = "PascalCase")]
pub enum Kind {
    #[default]
    Active,
    Passive,
    Toggle,
}
