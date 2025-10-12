use bevy::prelude::*;
use l2r_core::{
    crypt::session_keys::SessionKey,
    packets::{ClientPacketBuffer, L2rSerializeError},
};
use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Reflect)]
pub struct AuthLoginRequest {
    pub account: String,
    pub key: SessionKey,
}

impl fmt::Debug for AuthLoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AuthLoginRequest {{ account: {} }}", self.account)
    }
}

impl TryFrom<ClientPacketBuffer> for AuthLoginRequest {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let account = buffer.str()?;

        let key_bytes = buffer.bytes(16)?;

        // Sort the key bytes
        let mut sorted_key_bytes: [u8; 16] = [0; 16];

        sorted_key_bytes[..4].copy_from_slice(&key_bytes[8..12]);
        sorted_key_bytes[4..8].copy_from_slice(&key_bytes[12..16]);
        sorted_key_bytes[8..12].copy_from_slice(&key_bytes[4..8]);
        sorted_key_bytes[12..16].copy_from_slice(&key_bytes[..4]);

        let key = SessionKey::from(sorted_key_bytes);

        Ok(Self { account, key })
    }
}
