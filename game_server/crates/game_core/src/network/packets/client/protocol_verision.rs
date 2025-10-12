use crate::network::protocol;
use bevy::prelude::*;
use l2r_core::packets::{ClientPacketBuffer, L2rSerializeError};
use std::{
    convert::TryFrom,
    fmt::{self, Debug},
};

#[derive(Clone, PartialEq, Reflect)]
pub struct ClientProtocolVersion {
    pub protocol_version: protocol::Version,
}

impl Debug for ClientProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.protocol_version)
    }
}

impl TryFrom<ClientPacketBuffer> for ClientProtocolVersion {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        Ok(Self {
            protocol_version: protocol::Version::from(buffer.bytes(4)?),
        })
    }
}
