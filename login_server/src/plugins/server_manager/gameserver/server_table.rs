use super::GameServer;
use bevy::{log, platform::collections::HashSet, prelude::Resource};

#[derive(Clone, Default, Resource)]
pub struct GameServerTable(HashSet<GameServer>);

impl GameServerTable {
    pub fn register_game_server(&mut self, server: GameServer) {
        log::info!("Registering Game Server with id: {}", server.id());
        self.0.insert(server);
        log::info!("Current Game Server count: {}", self.0.len());
    }

    pub fn load_registered_game_servers(&self) -> Vec<GameServer> {
        self.0.iter().cloned().collect()
    }
}
