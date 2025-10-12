use bevy::{
    platform::collections::HashMap,
    prelude::{Deref, DerefMut, *},
};
use bevy_slinet::{ServerConfig, connection::ConnectionId, server::ServerConnection};

#[derive(Clone, Component, Copy, Debug, Deref, Eq, Hash, PartialEq, Reflect)]
pub struct SessionId(usize);
impl From<ConnectionId> for SessionId {
    fn from(id: ConnectionId) -> Self {
        SessionId(id.read())
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct ServerSessions(HashMap<SessionId, Entity>);
impl ServerSessions {
    pub fn get(&self, session_id: &SessionId) -> Result<Entity, BevyError> {
        self.0
            .get(session_id)
            .copied()
            .ok_or_else(|| BevyError::from(format!("SessionId not found {:?}", session_id)))
    }

    pub fn by_connection(&self, connection_id: &ConnectionId) -> Result<Entity, BevyError> {
        self.0
            .get(&SessionId::from(*connection_id))
            .copied()
            .ok_or_else(|| {
                BevyError::from(format!(
                    "SessionId not found for connection {:?}",
                    connection_id
                ))
            })
    }
}

// Marker Component to store game server sessions under entity
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct GameServerSessions;

pub trait L2rSession<T: ServerConfig> {
    fn new(connection: ServerConnection<T>) -> Self;
    fn id(&self) -> SessionId;
    fn access_level(&self) -> u8;
    fn send(&self, packet: T::ServerPacket);
    fn disconnect(&self);
}
