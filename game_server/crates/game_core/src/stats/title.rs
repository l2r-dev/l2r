use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Clone, Component, Debug, Default, Deref, DerefMut, Deserialize, PartialEq, Reflect, Serialize,
)]
pub struct NameTitle(Name);

impl NameTitle {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(Name::new(name))
    }
}
