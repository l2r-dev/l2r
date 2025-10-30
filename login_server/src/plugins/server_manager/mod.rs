use bevy::{
    log,
    prelude::{
        App, AssetEvent, AssetServer, Assets, EventReader, Handle, Plugin, Res, ResMut, Startup,
        Update,
    },
};
use bevy_common_assets::json::JsonAssetPlugin;
use std::net::Ipv4Addr;

mod gameserver;
pub use gameserver::*;

pub struct ServerManagerPlugin;

impl Plugin for ServerManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameServerTable>()
            .init_resource::<ServerNamesHandle>();

        app.add_plugins(JsonAssetPlugin::<ServerNames>::new(&["json"]));

        app.add_systems(
            Startup,
            (load_server_names_asset, register_default_game_servers),
        )
        .add_systems(Update, handle_server_names_loaded);
    }
}

fn load_server_names_asset(
    asset_server: Res<AssetServer>,
    mut server_names_handle: ResMut<ServerNamesHandle>,
) {
    let handle: Handle<ServerNames> = asset_server.load("server_names.json");
    server_names_handle.0 = handle;
}

fn handle_server_names_loaded(
    mut events: EventReader<AssetEvent<ServerNames>>,
    server_names_assets: Res<Assets<ServerNames>>,
    server_names_handle: Res<ServerNamesHandle>,
) {
    for event in events.read() {
        let id = server_names_handle.id();
        if event.is_loaded_with_dependencies(id)
            && let Some(server_names) = server_names_assets.get(id)
        {
            log::info!("Loaded {} server names.", server_names.len());
        }
    }
}

pub fn register_default_game_servers(mut server_table: ResMut<GameServerTable>) {
    use std::net::ToSocketAddrs;

    const DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    const DEFAULT_PORT: u32 = 7777;

    let server_addr =
        std::env::var("LS_GAME_SERVER_ADDR").unwrap_or_else(|_| DEFAULT_IP.to_string());
    let port_env = std::env::var("LS_GAME_SERVER_PORT").ok();
    let port: u32 = port_env
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // Try to resolve as IP, else as domain name
    let ip_addrs = if let Ok(ip) = server_addr.parse::<Ipv4Addr>() {
        vec![ip]
    } else {
        // Try to resolve domain name to IPv4 addresses
        let addr_port = format!("{server_addr}:{port}");
        match addr_port.to_socket_addrs() {
            Ok(iter) => {
                let ips: Vec<Ipv4Addr> = iter
                    .filter_map(|sockaddr| match sockaddr {
                        std::net::SocketAddr::V4(v4) => Some(*v4.ip()),
                        _ => None,
                    })
                    .collect();
                if ips.is_empty() {
                    log::error!(
                        "No IPv4 addresses found for domain '{}', using default {}",
                        server_addr,
                        DEFAULT_IP
                    );
                    vec![DEFAULT_IP]
                } else {
                    ips
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to resolve server address '{}': {}, using default {}",
                    server_addr,
                    e,
                    DEFAULT_IP
                );
                vec![DEFAULT_IP]
            }
        }
    };

    server_table.register_game_server(GameServer::new(
        9,
        true,
        ip_addrs,
        port,
        ServerType::Normal,
        false,
        5000,
    ));
}
