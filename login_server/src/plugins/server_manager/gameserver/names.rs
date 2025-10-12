use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Asset, Clone, Debug, Deref, DerefMut, Default, Deserialize, Resource, Serialize, TypePath,
)]
pub struct ServerNames(HashMap<u8, String>);

#[derive(Default, Deref, Resource)]
pub struct ServerNamesHandle(pub Handle<ServerNames>);
