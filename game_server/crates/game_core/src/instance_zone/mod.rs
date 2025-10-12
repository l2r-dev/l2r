use bevy::prelude::*;
use derive_more::{From, Into};

#[derive(Clone, Copy, Debug, Deref, Eq, From, Hash, Into, PartialEq, Reflect)]
pub struct InstanceZoneId(u32);
