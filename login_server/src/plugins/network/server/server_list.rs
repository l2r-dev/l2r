use super::LoginServerPacketCode;
use crate::plugins::server_manager::GameServerTable;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
#[derive(Clone)]
pub struct ServerListResponse {
    server_table: GameServerTable,
    last_server: u8,
    chars_on_servers: u8,
    chars_to_delete: u8,
}

impl std::fmt::Debug for ServerListResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{:?}> ServerListResponse [last_server: {}, chars_on_servers: {}, chars_to_delete: {}]",
            LoginServerPacketCode::SERVER_LIST_RESPONSE,
            self.last_server,
            self.chars_on_servers,
            self.chars_to_delete
        )
    }
}

impl L2rServerPacket for ServerListResponse {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::SERVER_LIST_RESPONSE.to_le_bytes());
        buffer.u8(self.server_table.load_registered_game_servers().len() as u8);
        buffer.u8(self.last_server);
        for server in self.server_table.load_registered_game_servers() {
            buffer.u8(server.id());
            buffer.extend(server.server_address().octets());
            buffer.u32(server.port());
            buffer.u8(server.age_limit().into());
            buffer.u8(server.is_pvp());
            buffer.u16(server.current_player_count());
            buffer.u16(server.max_players());
            buffer.u8(server.status().into());
            buffer.u32(server.server_type().into());
            buffer.bool(server.is_showing_brackets());
        }
        if self.chars_on_servers > 0 {
            buffer.u8(self.chars_on_servers);
            buffer.u8(self.chars_to_delete);
        }
        buffer
    }
}
impl ServerListResponse {
    pub fn new(
        server_table: GameServerTable,
        last_server: u8,
        chars_on_servers: u8,
        chars_to_delete: u8,
    ) -> Self {
        Self {
            server_table,
            last_server,
            chars_on_servers,
            chars_to_delete,
        }
    }
}
