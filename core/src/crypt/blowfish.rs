use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const GAME_SERVER_STATIC_KEY: [u8; 8] = 0x_97_31_6C_A1_01_93_27_C8_u64.to_le_bytes();

/// 128-bit (16-byte) key for Blowfish encryption.
/// Used in [`SessionKey`], [`KeyPacket`], [`GameServerKeyPair`], etc.
#[derive(Clone, Copy, Deref, Deserialize, Eq, PartialEq, Reflect, Serialize)]
pub struct BlowfishKey([u8; 16]);

impl Debug for BlowfishKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print as a hex string
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl BlowfishKey {
    /// Generates a random 16-byte key using `rand::Rng`.
    pub fn new() -> BlowfishKey {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        BlowfishKey(bytes)
    }

    /// Returns a fixed key (for tests and initial communication).
    pub fn fixed() -> BlowfishKey {
        Self(0x_6C_6C_6C_6C_55_6C_2B_CC_B1_90_CE_82_5B_CB_60_6B_i128.to_le_bytes())
    }

    pub fn to_le_bytes(&self) -> [u8; 16] {
        self.0
    }

    /// Shifts the key by a given size. This modifies bytes 8..12 as a little-endian i32.
    pub fn shift_key(&mut self, size: usize) {
        // Interpret bytes 8..12 as a little-endian i32
        let mut temp = [0u8; 4];
        temp.copy_from_slice(&self.0[8..12]);
        let mut old = i32::from_le_bytes(temp);
        old += size as i32;
        self.0[8..12].copy_from_slice(&old.to_le_bytes());
    }

    /// Generates a new game server key, but sets bytes 8..16 to a specific constant (that pre-defined on client).
    pub fn new_game_server_key() -> BlowfishKey {
        let mut key = BlowfishKey::new();
        key.0[8..16].copy_from_slice(&GAME_SERVER_STATIC_KEY);
        key
    }
}

impl Default for BlowfishKey {
    fn default() -> Self {
        BlowfishKey::new()
    }
}

impl From<[u8; 16]> for BlowfishKey {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}
