use super::{GameServerPacket, GameServerPacketCodes};
use crate::network::session::GameServerSession;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Default, Reflect)]
pub struct NetPing;
impl NetPing {
    pub fn ping(
        time: Res<Time>,
        mut last_time: Local<f32>,
        mut sessions: Query<(Entity, Mut<GameServerSession>)>,
        mut commands: Commands,
    ) {
        if time.elapsed_secs() - *last_time >= 90.0 {
            *last_time = time.elapsed_secs();
            let ping = GameServerPacket::NetPing(NetPing);
            for (entity, mut session) in sessions.iter_mut() {
                session.start_ping();
                commands.trigger_targets(ping.clone(), entity);
            }
        }
    }
}
impl L2rServerPacket for NetPing {
    fn buffer(self) -> ServerPacketBuffer {
        GameServerPacketCodes::NET_PING.to_le_bytes().into()
    }
}
