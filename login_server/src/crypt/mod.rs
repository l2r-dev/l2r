use crate::plugins::network::{
    client::{LoginClientPacket, auth_login::AuthLoginRequest},
    server::LoginServerPacket,
};
use bevy::log;
use blowfish::{
    BlowfishLE,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
};
use generic_array::GenericArray;
use l2r_core::{
    crypt::{
        blowfish::BlowfishKey, crypt_engine::CryptEngine, scrambled_rsa::ScrambledRsaKey,
        xor::XorCrypt,
    },
    packets::{
        ClientPacketBuffer, L2rSerializeError, L2rServerPacket, L2rServerPackets,
        ServerPacketBuffer,
    },
    utils::log_trace_byte_table,
};
use std::{
    fmt::{Debug, Formatter},
    sync::LazyLock,
};

mod checksum;

use checksum::*;

// Static blowfish crypt used for InitPacket and tests
pub static STATIC_BLOWFISH_CRYPT: LazyLock<BlowfishLE> =
    LazyLock::new(|| BlowfishLE::new_from_slice(&BlowfishKey::fixed().to_le_bytes()).unwrap());

#[derive(Clone)]
pub struct LoginCryptParts {
    pub rsa_key: ScrambledRsaKey,
    pub blowfish_key: BlowfishKey,
    pub blowfish_crypt: BlowfishLE,
    pub xor_crypt: XorCrypt,
}

// Default implementation used in real encryption
impl Default for LoginCryptParts {
    fn default() -> Self {
        let blowfish_key = BlowfishKey::new();
        let blowfish_crypt = BlowfishLE::new_from_slice(&blowfish_key.to_le_bytes()).unwrap();
        Self {
            rsa_key: Default::default(),
            blowfish_key,
            blowfish_crypt,
            xor_crypt: Default::default(),
        }
    }
}

// Fixed implementation used in tests
impl LoginCryptParts {
    #[cfg(test)]
    fn fixed() -> Self {
        let blowfish_key = BlowfishKey::fixed();
        let blowfish_crypt = BlowfishLE::new_from_slice(&blowfish_key.to_le_bytes()).unwrap();
        Self {
            rsa_key: ScrambledRsaKey::fixed(),
            blowfish_key,
            blowfish_crypt,
            xor_crypt: XorCrypt::new(l2r_core::crypt::xor::XorKey::from(666)),
        }
    }
}

/// The LoginCryptEngine is responsible for encrypting and decrypting packets
/// exchanged between the client and server during the login process.
#[derive(Clone, Default)]
pub struct LoginCryptEngine(LoginCryptParts);

impl Debug for LoginCryptEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginCryptEngine")
            .field("crypt", &"Blowfish instance")
            .finish()
    }
}

impl LoginCryptEngine {
    /// Creates fixed instance of LoginCryptEngine to be used in tests.
    #[cfg(test)]
    pub fn fixed() -> Self {
        Self(LoginCryptParts::fixed())
    }

    /// Reads UTF-8 string from bytes until the first null byte (0x00).
    pub fn read_string_from_bytes(bytes: &[u8]) -> Result<String, L2rSerializeError> {
        match bytes.iter().position(|&b| b == 0) {
            Some(null_pos) => String::from_utf8(bytes[..null_pos].to_vec()).map_err(|e| {
                L2rSerializeError::with_source(
                    "Failed to parse UTF-8 string".to_string(),
                    e,
                    bytes.to_vec(),
                )
            }),
            None => Err(L2rSerializeError::InvalidString),
        }
    }

    /// Appends padding to the provided byte vector to ensure its length is a multiple of the block size (8 bytes).
    fn append_padding(bytes: &mut ServerPacketBuffer) {
        let size = bytes.len();
        if !size.is_multiple_of(Self::BLOCK_SIZE) {
            let padding = Self::BLOCK_SIZE - (size % Self::BLOCK_SIZE);
            bytes.resize_with(size + padding, Default::default);
        }
    }

    /// Removes padding from the provided byte vector if the last 4 bytes are all zero.
    fn remove_padding(bytes: &mut ClientPacketBuffer) {
        let size = bytes.len();
        if size < 4 {
            return;
        }
        let padding = bytes[size - 4..].iter().all(|&b| b == 0);
        if padding {
            bytes.truncate(size - 4);
        }
    }

    /// Computes and appends a checksum to the provided byte vector.
    fn append_checksum(bytes: &mut ServerPacketBuffer) {
        let checksum = CheckSum::from(bytes.as_slice());
        bytes.extend(checksum.to_le_bytes());
    }

    /// Verifies the checksum of the provided data.
    ///
    /// The input slice `data_with_checksum` is expected to contain the original data
    /// followed by a 4-byte checksum at the end. This function computes the checksum
    /// of the data portion and compares it to the checksum value appended at the end.
    ///
    /// Returns `true` if the checksum matches, `false` otherwise.
    fn verify_checksum(data_with_checksum: &[u8]) -> bool {
        // The data must be at least 4 bytes to contain a checksum.
        let total_len = data_with_checksum.len();
        if total_len < 4 {
            return false;
        }
        // Split the slice into the data and the checksum part.
        let data_len = total_len - 4;
        let data = &data_with_checksum[..data_len];
        // Compute the expected checksum from the data.
        let expected_checksum = CheckSum::from(data);
        // Extract the actual checksum bytes from the end of the slice.
        let checksum_bytes: [u8; 4] = data_with_checksum[data_len..]
            .try_into()
            .unwrap_or_default();
        // Convert the checksum bytes into a CheckSum value.
        let actual_checksum = CheckSum::from(checksum_bytes.as_ref());
        // Return true if the checksums match, false otherwise.
        actual_checksum == expected_checksum
    }
}

impl CryptEngine<LoginClientPacket, LoginServerPacket> for LoginCryptEngine {
    fn encrypt(&mut self, packet: LoginServerPacket) -> Result<Vec<u8>, L2rSerializeError> {
        let crypt_parts = &self.0;

        let blowfish_crypt = match packet {
            LoginServerPacket::InitPacket(_) => &STATIC_BLOWFISH_CRYPT,
            _ => &crypt_parts.blowfish_crypt,
        };

        let mut buffer = match packet {
            LoginServerPacket::InitPacket(p) => {
                let mut buffer = p.build(crypt_parts.clone()).buffer();
                log_trace_byte_table(&buffer, "InitPacket");
                Self::append_padding(&mut buffer);
                let key = crypt_parts.xor_crypt.encode(&mut buffer);
                buffer.extend(key);
                log_trace_byte_table(&buffer, "After xor crypt:");
                buffer
            }
            _ => {
                let mut buffer = packet.buffer();
                log_trace_byte_table(&buffer, "Original");
                Self::append_checksum(&mut buffer);
                log_trace_byte_table(&buffer, "After checksum:");
                buffer
            }
        };

        // Perform encryption on the prepared byte array
        Self::append_padding(&mut buffer);
        for chunk in buffer.chunks_exact_mut(8) {
            blowfish_crypt.encrypt_block(GenericArray::from_mut_slice(chunk));
        }

        log_trace_byte_table(&buffer, "Encrypted");
        Ok(buffer.into())
    }

    fn decrypt(&mut self, packet: &[u8]) -> Result<LoginClientPacket, L2rSerializeError> {
        let mut buffer = ClientPacketBuffer::new(packet);
        // Iterate through chunks of bytes and decrypt them in-place
        for chunk in buffer.chunks_exact_mut(8) {
            self.0
                .blowfish_crypt
                .decrypt_block(GenericArray::from_mut_slice(chunk));
        }
        log_trace_byte_table(&buffer, "Decrypted");

        if buffer[0] == 0x00 {
            log_trace_byte_table(&buffer, "After XOR");
            let crypt_parts = &self.0;
            return Ok(LoginClientPacket::AuthLogin(
                AuthLoginRequest::try_from(buffer)?.decode(crypt_parts)?,
            ));
        }
        Self::remove_padding(&mut buffer);
        let check_result = LoginCryptEngine::verify_checksum(&buffer);
        if !check_result {
            let msg = "Checksum verification failed!";
            log::error!(msg);
            return Err(L2rSerializeError::from(msg));
        }
        Self::remove_padding(&mut buffer);
        LoginClientPacket::try_from(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::network::{
        LoginServerProtocol,
        server::{init::InitPacket, login_ok::LoginOk},
    };
    use bevy_slinet::connection::ConnectionId;
    use l2r_core::crypt::session_keys::SessionKey;

    #[test]
    fn test_login_encryption_init_packet() {
        let mut crypt_engine = LoginCryptEngine::fixed();
        let session_id = ConnectionId::next().into();
        let protocol_version = LoginServerProtocol::NewProtocolVersion;
        let init_packet = InitPacket::new(session_id, protocol_version);
        let lsp = LoginServerPacket::InitPacket(Box::new(init_packet));
        let original_bytes = [
            202, 235, 207, 13, 125, 6, 76, 120, 148, 37, 199, 197, 250, 36, 106, 192, 223, 44, 100,
            44, 99, 255, 220, 223, 23, 200, 250, 117, 230, 124, 121, 204, 124, 107, 221, 116, 203,
            229, 233, 137, 18, 86, 110, 217, 213, 250, 62, 158, 238, 100, 65, 40, 140, 244, 185,
            242, 204, 210, 36, 55, 211, 229, 64, 129, 49, 7, 71, 157, 43, 30, 91, 160, 131, 139,
            108, 47, 19, 209, 132, 103, 58, 227, 189, 196, 180, 248, 169, 111, 66, 180, 119, 93,
            40, 154, 1, 97, 248, 18, 138, 103, 3, 200, 136, 166, 96, 43, 123, 201, 234, 67, 119,
            205, 1, 4, 96, 90, 200, 34, 243, 51, 235, 38, 202, 173, 239, 153, 145, 64, 73, 58, 104,
            159, 187, 60, 145, 116, 83, 176, 175, 89, 241, 86, 166, 231, 58, 57, 16, 124, 100, 69,
            189, 107, 6, 185, 243, 69, 47, 65, 30, 94, 25, 63, 32, 20, 255, 185, 218, 74, 11, 32,
            180, 114, 0, 41, 95, 112, 170, 198, 27, 107, 201, 202, 243, 77,
        ]
        .to_vec();
        let encrypted_bytes = crypt_engine.encrypt(lsp).unwrap();
        assert_eq!(original_bytes, encrypted_bytes);
    }
    #[test]
    fn test_login_encryption_login_ok_packet() {
        let mut crypt_engine = LoginCryptEngine::fixed();
        let session_key = SessionKey::from(BlowfishKey::fixed().to_le_bytes());
        let login_ok_packet = LoginOk::new(session_key);
        let lsp = LoginServerPacket::LoginOk(login_ok_packet);
        let original_bytes = [
            178, 170, 131, 11, 119, 194, 113, 162, 157, 191, 131, 13, 127, 214, 28, 173, 130, 59,
            252, 19, 148, 24, 161, 131, 70, 214, 161, 155, 128, 133, 71, 70, 70, 214, 161, 155,
            128, 133, 71, 70, 223, 104, 174, 49, 107, 129, 195, 164, 198, 129, 165, 6, 198, 126,
            229, 249,
        ]
        .to_vec();
        let encrypted_bytes = crypt_engine.encrypt(lsp).unwrap();
        assert_eq!(original_bytes, encrypted_bytes);
    }

    #[test]
    fn xor_crypt_encode_decode() {
        let crypt = XorCrypt::default();

        let data = b"Hello, this is a test message!".to_vec();
        let mut data = ServerPacketBuffer::from(data);
        let mut original_data = data.clone();
        LoginCryptEngine::append_padding(&mut data);
        LoginCryptEngine::append_padding(&mut original_data);

        let key = crypt.encode(data.as_mut());
        data.extend(key);
        assert_ne!(data, original_data);

        XorCrypt::decode(data.as_mut());
        // Remove the key that was appended during encoding
        let truncate_length = data.len() - 4;
        data.truncate(truncate_length);
        assert_eq!(data, original_data);
    }

    #[test]
    fn test_checksum_data_slice() {
        let data = b"test_data";
        let sum = 1179947;
        let expected_checksum = CheckSum::new(sum);
        let checksum = CheckSum::from(data.as_slice());
        assert_eq!(expected_checksum, checksum);
    }

    #[test]
    fn test_append_checksum() {
        let data = b"test_data".to_vec();
        let mut data = ServerPacketBuffer::from(data);
        let expected_data = vec![116, 101, 115, 116, 95, 100, 97, 116, 97, 43, 1, 18, 0];
        let expected_data = ServerPacketBuffer::from(expected_data);
        LoginCryptEngine::append_checksum(&mut data);
        assert_eq!(expected_data, data);
        assert!(LoginCryptEngine::verify_checksum(&data));
    }

    #[test]
    fn test_verify_checksum() {
        let data_with_checksum = b"test_data\x2B\x01\x12\x00";
        assert!(LoginCryptEngine::verify_checksum(data_with_checksum));
    }

    #[test]
    fn test_verify_checksum2() {
        let raw_data = b"\x00\x00\x07\x00\x00\x00\x00\x23\x01\x00\x00\x67\x45\x00\x00\xAB\x89\x00\x00\xEF\xCD\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let data_vec = raw_data.to_vec();
        let mut data = ServerPacketBuffer::from(data_vec.clone());
        LoginCryptEngine::append_checksum(&mut data);
        assert!(LoginCryptEngine::verify_checksum(&data));
    }
}
