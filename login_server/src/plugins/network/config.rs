use crate::{
    crypt::LoginCryptEngine,
    plugins::network::{client::LoginClientPacket, server::LoginServerPacket},
};
use bevy_slinet::{ServerConfig, protocols::tcp::TcpProtocol, serializer::SerializerAdapter};
use l2r_core::packets::{L2rLenSerializer, L2rSerializeError, L2rSerializer};
use std::sync::{Arc, Mutex};

pub struct LoginServerNetworkConfig;

impl ServerConfig for LoginServerNetworkConfig {
    type ClientPacket = LoginClientPacket;
    type ServerPacket = LoginServerPacket;
    type Protocol = TcpProtocol;
    type SerializerError = L2rSerializeError;
    fn build_serializer()
    -> SerializerAdapter<Self::ClientPacket, Self::ServerPacket, Self::SerializerError> {
        SerializerAdapter::Mutable(Arc::new(Mutex::new(L2rSerializer::<
            LoginCryptEngine,
            Self::ClientPacket,
            Self::ServerPacket,
        >::new(
            LoginCryptEngine::default()
        ))))
    }
    type LengthSerializer = L2rLenSerializer;
}
