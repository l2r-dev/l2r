use crate::{
    crypt::GameCryptEngine,
    network::packets::{client::GameClientPacket, server::GameServerPacket},
};
use bevy_slinet::{ServerConfig, protocols::tcp::TcpProtocol, serializer::SerializerAdapter};
use l2r_core::packets::{L2rLenSerializer, L2rSerializeError, L2rSerializer};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct GameServerNetworkConfig;

impl ServerConfig for GameServerNetworkConfig {
    type ClientPacket = GameClientPacket;
    type ServerPacket = GameServerPacket;
    type Protocol = TcpProtocol;
    type SerializerError = L2rSerializeError;
    fn build_serializer()
    -> SerializerAdapter<Self::ClientPacket, Self::ServerPacket, Self::SerializerError> {
        SerializerAdapter::Mutable(Arc::new(Mutex::new(L2rSerializer::<
            GameCryptEngine,
            Self::ClientPacket,
            Self::ServerPacket,
        >::new(
            GameCryptEngine::default()
        ))))
    }
    type LengthSerializer = L2rLenSerializer;
}
