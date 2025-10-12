use crate::packets::L2rSerializeError;

/// Trait for cryptographic engines, used in [`LoginCryptEngine`] and [`GameCryptEngine`].
pub trait CryptEngine<ReceivingPacket, SendingPacket>: Default + Send + Sync + 'static {
    const BLOCK_SIZE: usize = 8;
    fn encrypt(&mut self, packet: SendingPacket) -> Result<Vec<u8>, L2rSerializeError>;

    fn decrypt(&mut self, packet: &[u8]) -> Result<ReceivingPacket, L2rSerializeError>;
}
