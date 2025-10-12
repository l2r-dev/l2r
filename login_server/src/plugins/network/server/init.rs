use super::LoginServerPacketCode;
use crate::{crypt::LoginCryptParts, plugins::network::LoginServerProtocol};
use l2r_core::{
    model::session::SessionId,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

const UNKNOWN_BYTES: [u8; 16] = [
    0x29, 0xDD, 0x95, 0x4E, 0x77, 0xC3, 0x9C, 0xFC, 0x97, 0xAD, 0xB6, 0x20, 0x07, 0xBD, 0xE0, 0xF7,
];

#[derive(Clone)]
pub struct InitPacket {
    session_id: SessionId,
    protocol_version: LoginServerProtocol,
    crypt_parts: Option<LoginCryptParts>,
}

impl std::fmt::Debug for InitPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{:?}> InitPacket [session_id: {:?}, protocol_version: {:?}]",
            LoginServerPacketCode::INIT_PACKET,
            self.session_id,
            self.protocol_version
        )
    }
}

impl L2rServerPacket for InitPacket {
    fn buffer(self) -> ServerPacketBuffer {
        let Some(crypt_parts) = self.crypt_parts else {
            bevy::log::error!("Crypt parts not found for InitPacket");
            return ServerPacketBuffer::new();
        };
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::INIT_PACKET.to_le_bytes());
        if self.protocol_version == LoginServerProtocol::OldProtocolVersion {
            buffer.u32_from_usize(*self.session_id);
            buffer.u32(self.protocol_version.into());
        }
        if self.protocol_version == LoginServerProtocol::NewProtocolVersion {
            buffer.u32_from_usize(*self.session_id);
            buffer.u32(self.protocol_version.into());
            buffer.extend_from_slice(&crypt_parts.rsa_key.scramble());
            buffer.extend_from_slice(&UNKNOWN_BYTES);
            buffer.extend_from_slice(&crypt_parts.blowfish_key.to_le_bytes());
            buffer.u8(0x00);
        }
        buffer
    }
}
impl InitPacket {
    pub fn new(session_id: SessionId, protocol_version: LoginServerProtocol) -> Self {
        Self {
            session_id,
            protocol_version,
            crypt_parts: None,
        }
    }
    pub fn build(mut self, crypt_parts: LoginCryptParts) -> Self {
        self.crypt_parts = Some(crypt_parts);
        self
    }
}
