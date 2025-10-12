use super::packets::server::{GameServerPacket, GameServerPackets};
use bevy::prelude::*;

#[derive(Clone, Debug, Reflect)]
pub enum BroadcastScope {
    All,
    Radius(f32),
    Known,
    KnownAndSelf,
    InRegion,
    Entities(Vec<Entity>),
}

#[derive(Debug, Event)]
pub struct ServerPacketBroadcast {
    pub packet: GameServerPacket,
    pub scope: BroadcastScope,
}

impl ServerPacketBroadcast {
    pub fn new(packet: GameServerPacket) -> Self {
        Self {
            packet,
            scope: BroadcastScope::KnownAndSelf,
        }
    }
}

#[derive(Debug, Event)]
pub struct ServerPacketsBroadcast {
    pub packets: GameServerPackets,
    pub scope: BroadcastScope,
}

impl ServerPacketsBroadcast {
    pub fn new(packets: GameServerPackets) -> Self {
        Self {
            packets,
            scope: BroadcastScope::KnownAndSelf,
        }
    }
}
