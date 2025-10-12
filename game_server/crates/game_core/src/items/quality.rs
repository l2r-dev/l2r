use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum Quality {
    Common,
    Shadow,
    #[default]
    Normal,
    Rare,
    Masterwork,
}
