use bevy::prelude::*;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Component,
    Copy,
    Debug,
    Default,
    Deref,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Reflect,
    From,
    Into,
)]
pub struct DeleteTimer(u32);
