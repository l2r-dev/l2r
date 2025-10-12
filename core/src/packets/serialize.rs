use super::{L2rClientPacket, L2rServerPackets};
use crate::{crypt::crypt_engine::CryptEngine, utils::log_trace_byte_table};
use bevy_slinet::{
    packet_length_serializer::{
        PacketLengthDeserializationError, PacketLengthSerializer, PacketTooLargeError,
    },
    serializer::MutableSerializer,
};
use num_enum::TryFromPrimitiveError;
use std::{error::Error, marker::PhantomData};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum L2rSerializeError {
    #[error("{message}: {}", crate::utils::format_bytes_hex(bytes.as_ref()))]
    Generic {
        message: String,
        #[source]
        source: Option<Box<dyn Error + Send + Sync>>,
        bytes: Vec<u8>,
    },

    #[error("Failed to convert from primitive: {0}")]
    PrimitiveConversion(String),

    #[error("Failed to convert from slice: {0}")]
    SliceConversion(#[from] std::array::TryFromSliceError),

    #[error("Failed to convert from UTF-16: {0}")]
    Utf16Conversion(#[from] std::string::FromUtf16Error),

    #[error("Invalid packet format: {message}")]
    InvalidPacket { message: String, bytes: Vec<u8> },

    #[error("Not enough bytes")]
    NotEnoughBytes,

    #[error("Invalid String")]
    InvalidString,
}

impl L2rSerializeError {
    pub fn new(message: String, bytes: &[u8]) -> Self {
        Self::Generic {
            message,
            source: None,
            bytes: bytes.to_vec(),
        }
    }

    pub fn with_source<E: Error + Send + Sync + 'static>(
        message: String,
        source: E,
        bytes: Vec<u8>,
    ) -> Self {
        Self::Generic {
            message,
            source: Some(Box::new(source)),
            bytes,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Generic { bytes, .. } => bytes.as_ref(),
            Self::InvalidPacket { bytes, .. } => bytes.as_ref(),
            _ => &[],
        }
    }
}

impl<T> From<TryFromPrimitiveError<T>> for L2rSerializeError
where
    T: num_enum::TryFromPrimitive,
{
    fn from(err: TryFromPrimitiveError<T>) -> Self {
        L2rSerializeError::PrimitiveConversion(format!("{err:?}"))
    }
}

impl From<&str> for L2rSerializeError {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
            source: None,
            bytes: vec![],
        }
    }
}

#[derive(Clone, Default)]
pub struct L2rSerializer<C, SendingPacket, ReceivingPacket>
where
    C: CryptEngine<SendingPacket, ReceivingPacket>,
{
    crypt_engine: C,
    _client: PhantomData<ReceivingPacket>,
    _server: PhantomData<SendingPacket>,
}
impl<C: CryptEngine<ReceivingPacket, SendingPacket>, SendingPacket, ReceivingPacket>
    L2rSerializer<C, ReceivingPacket, SendingPacket>
where
    C: CryptEngine<ReceivingPacket, SendingPacket>,
{
    pub fn new(crypt_engine: C) -> Self {
        Self {
            crypt_engine,
            _client: PhantomData,
            _server: PhantomData,
        }
    }
}

impl<ReceivingPacket, SendingPacket, C> MutableSerializer<ReceivingPacket, SendingPacket>
    for L2rSerializer<C, ReceivingPacket, SendingPacket>
where
    C: CryptEngine<ReceivingPacket, SendingPacket>,
    ReceivingPacket: L2rClientPacket,
    SendingPacket: L2rServerPackets,
{
    type Error = L2rSerializeError;

    /// Serializes a packet into a byte vector.
    fn serialize(&mut self, packet: SendingPacket) -> Result<Vec<u8>, Self::Error> {
        log::trace!("[S->C]: {packet:?}");
        self.crypt_engine.encrypt(packet)
    }

    /// Deserializes a packet from a byte slice
    fn deserialize(&mut self, buffer: &[u8]) -> Result<ReceivingPacket, Self::Error> {
        log_trace_byte_table(buffer, "Deserializing packet");
        let packet = self.crypt_engine.decrypt(buffer);
        log::trace!("[C->S]: {packet:?}");
        packet
    }
}

#[derive(Default)]
pub struct L2rLenSerializer;

impl PacketLengthSerializer for L2rLenSerializer {
    type Error = PacketTooLargeError;

    const SIZE: usize = 2;

    fn serialize_packet_length(&self, length: usize) -> Result<Vec<u8>, Self::Error> {
        if length > u16::MAX as usize {
            return Err(PacketTooLargeError {
                length,
                max_length: u16::MAX as usize,
            });
        }
        let total_length = length + 2;
        let length_bytes = (total_length as u16).to_le_bytes();
        Ok(length_bytes.to_vec())
    }

    fn deserialize_packet_length(
        &self,
        buffer: &[u8],
    ) -> Result<usize, PacketLengthDeserializationError<Self::Error>> {
        if buffer.len() > u16::MAX as usize {
            return Err(PacketLengthDeserializationError::Err(PacketTooLargeError {
                length: buffer.len(),
                max_length: u16::MAX as usize,
            }));
        }

        let arr: [u8; 2] = buffer[..2]
            .try_into()
            .map_err(|_| PacketLengthDeserializationError::NeedMoreBytes(2 - buffer.len()))?;

        let length = u16::from_le_bytes(arr) as usize;
        Ok(length - 2)
    }
}
