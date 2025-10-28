use crate::network::packets::{client::GameClientPacket, server::GameServerPacket};
use l2r_core::{
    crypt::{blowfish::BlowfishKey, crypt_engine::CryptEngine},
    packets::{ClientPacketBuffer, L2rSerializeError, L2rServerPacket, L2rServerPackets},
    utils::log_trace_byte_table,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct GameServerKeyPair(BlowfishKey, BlowfishKey);
impl Default for GameServerKeyPair {
    fn default() -> Self {
        let key = BlowfishKey::new_game_server_key();
        Self(key, key)
    }
}

#[derive(Clone, Default)]
pub struct GameCryptEngine(GameServerKeyPair);

impl fmt::Debug for GameCryptEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameCryptEngine")
            .field("in_key", &self.get_in_key())
            .field("out_key", &self.get_out_key())
            .finish()
    }
}

impl GameCryptEngine {
    pub fn read_string_from_bytes(bytes: &[u8]) -> Result<String, L2rSerializeError> {
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|&char| char != 0)
            .collect();
        String::from_utf16(&utf16).map_err(L2rSerializeError::from)
    }

    pub fn get_in_key(&self) -> BlowfishKey {
        self.0.0
    }

    pub fn get_out_key(&self) -> BlowfishKey {
        self.0.1
    }

    pub fn shift_in_key(&mut self, shift: usize) {
        self.0.0.shift_key(shift);
    }

    pub fn shift_out_key(&mut self, shift: usize) {
        self.0.1.shift_key(shift);
    }
}

impl CryptEngine<GameClientPacket, GameServerPacket> for GameCryptEngine {
    fn encrypt(&mut self, mut packet: GameServerPacket) -> Result<Vec<u8>, L2rSerializeError> {
        packet = match packet {
            GameServerPacket::KeyPacket(p) => {
                let bytes = p.build(self.get_out_key()).buffer();
                #[cfg(debug_assertions)]
                log_trace_byte_table(&bytes, "KeyPacket");
                return Ok(bytes.to_vec());
            }
            _ => packet,
        };

        let mut buffer = packet.buffer();

        #[cfg(debug_assertions)]
        log_trace_byte_table(&buffer, "Raw Server Packet");

        let mut acc = 0;
        let out_key = self.get_out_key().to_le_bytes();

        for i in 0..buffer.len() {
            let curr_byte = buffer[i] as i8;
            let key_byte = out_key[i & 15] as i8;
            let encrypted_byte = curr_byte ^ key_byte ^ acc;
            acc = encrypted_byte;
            buffer[i] = encrypted_byte as u8;
        }

        #[cfg(debug_assertions)]
        log_trace_byte_table(&buffer, "Encrypted");

        self.shift_out_key(buffer.len());
        Ok(buffer.into())
    }

    fn decrypt(&mut self, packet: &[u8]) -> Result<GameClientPacket, L2rSerializeError> {
        if packet[0] == 0x0E {
            return GameClientPacket::try_from(ClientPacketBuffer::new(packet));
        }
        let mut buffer = ClientPacketBuffer::new(packet);

        let mut prev_byte: i8 = 0;
        let in_key_bytes = self.get_in_key().to_le_bytes();

        for i in 0..buffer.len() {
            let curr_byte = buffer[i] as i8;
            let key_byte = in_key_bytes[i & 15] as i8;
            let decrypted_byte = curr_byte ^ key_byte ^ prev_byte;
            prev_byte = curr_byte;
            buffer[i] = decrypted_byte as u8;
        }

        self.shift_in_key(buffer.len());

        GameClientPacket::try_from(buffer)
    }
}
