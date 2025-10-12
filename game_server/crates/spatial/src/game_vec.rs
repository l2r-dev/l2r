use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use serde::{Deserialize, Serialize};

/// Represents a 3D vector with integer coordinates, used for client-side world positioning.
/// Bevy's [`Vec3`] <> [`GameVec3`] <> [`GeoVec3`].
/// Note: Y is up in Bevy, Z is up in GameVec3.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub struct GameVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl std::fmt::Display for GameVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl TryFrom<&mut ClientPacketBuffer> for GameVec3 {
    type Error = L2rSerializeError;
    fn try_from(value: &mut ClientPacketBuffer) -> Result<Self, L2rSerializeError> {
        let x = value.i32()?;
        let y = value.i32()?;
        let z = value.i32()?;
        Ok(GameVec3 { x, y, z })
    }
}

impl GameVec3 {
    /// Creates a new `GameVec3` instance.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        GameVec3 { x, y, z }
    }

    pub fn to_le_bytes(&self) -> [u8; 12] {
        let mut bytes = [0; 12];
        bytes[0..4].copy_from_slice(&self.x.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.y.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.z.to_le_bytes());
        bytes
    }
}

impl From<Vec3> for GameVec3 {
    fn from(value: Vec3) -> Self {
        // Convert Bevy's Vec3 to GameVec3
        // Note that Bevy's Y is up, while GameVec3's Z is up
        GameVec3 {
            x: value.x as i32,
            y: value.z as i32, // Bevy's Z becomes GameVec3's Y
            z: value.y as i32, // Bevy's Y becomes GameVec3's Z
        }
    }
}

impl From<GameVec3> for Vec3 {
    fn from(value: GameVec3) -> Self {
        // Convert GameVec3 to Bevy's Vec3
        // Note that GameVec3's Z is up, while Bevy's Y is up
        Vec3::new(
            value.x as f32,
            value.z as f32, // GameVec3's Z becomes Bevy's Y
            value.y as f32, // GameVec3's Y becomes Bevy's Z
        )
    }
}
