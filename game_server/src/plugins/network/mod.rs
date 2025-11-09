use crate::plugins::network::{
    broadcast::NetworkBroadcastPlugin,
    functions::{
        send_char_info_handler, send_char_selection_info, send_multiple_server_packets,
        send_server_packet, send_user_info_handler, send_user_info_when_updated,
    },
    scripting::{ClientPacketScriptingPlugin, GameServerPacketScriptingPlugin},
};
use bevy::{log, prelude::*};
use bevy_slinet::{
    connection::MaxPacketSize,
    server::{DisconnectionEvent, NewConnectionEvent, PacketReceiveEvent, ServerPlugin},
};
use game_core::{
    account::Account,
    character::{self, CharacterSave},
    network::{
        GameNetworkMetric,
        config::GameServerNetworkConfig,
        packets::{
            GameServerPacketMetric,
            server::{GameServerPacketPlugin, NetPing},
        },
        session::GameServerSession,
    },
};
use l2r_core::{
    metrics::{Metrics, MetricsAppExt},
    model::session::{GameServerSessions, L2rSession, ServerSessions, SessionId},
    plugins::custom_hierarchy::DespawnChildOf,
};
use state::GameServerStateSystems;

pub mod broadcast;
pub mod functions;

pub mod scripting;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MaxPacketSize(65535));

        app.add_observer(send_char_info_handler)
            .add_observer(send_user_info_handler)
            .add_observer(send_char_selection_info)
            .add_observer(send_server_packet)
            .add_observer(send_multiple_server_packets)
            .add_observer(Self::new_connection)
            .add_observer(Self::disconnection_handler);

        app.add_plugins(ServerPlugin::<GameServerNetworkConfig>::bind(
            "0.0.0.0:7777",
        ))
        .add_plugins(GameClientPacketPlugin)
        .add_plugins(GameServerPacketPlugin)
        .add_plugins(NetworkBroadcastPlugin);

        app.add_plugins((GameServerPacketScriptingPlugin, ClientPacketScriptingPlugin));

        app.add_systems(Update, send_user_info_when_updated);

        // Register network metrics
        app.register_counter(GameNetworkMetric::PacketsReceived, "Total packets received")
            .register_counter(GameNetworkMetric::NewConnections, "New connections count")
            .register_gauge(GameNetworkMetric::ActiveSessions, "Active sessions count");

        app.register_counter(GameServerPacketMetric::PacketsSent, "Total packets sent");

        app.init_resource::<ServerSessions>()
            .add_systems(Startup, Self::spawn_server_sessions_entity);

        app.add_systems(Update, NetPing::ping);
    }
}

struct GameClientPacketPlugin;
impl Plugin for GameClientPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(increase_packet_count);
    }
}

fn increase_packet_count(
    _: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    metrics: Res<Metrics>,
) -> Result<()> {
    metrics.counter(GameNetworkMetric::PacketsReceived)?.inc();
    Ok(())
}

impl NetworkPlugin {
    fn new_connection(
        new_connection: Trigger<NewConnectionEvent<GameServerNetworkConfig>>,
        mut commands: Commands,
        mut sessions: ResMut<ServerSessions>,
        server_sessions_list: Query<Entity, With<GameServerSessions>>,
        metrics: Res<Metrics>,
        config: Res<::config::Config>,
        state: Res<State<GameServerStateSystems>>,
    ) -> Result<()> {
        if *state.get() != GameServerStateSystems::Run {
            new_connection.event().connection.disconnect();
            return Ok(());
        }
        log::debug!(
            "New connection from {}",
            new_connection.event().connection.peer_addr()
        );

        metrics.counter(GameNetworkMetric::PacketsReceived)?.inc();
        metrics.counter(GameNetworkMetric::NewConnections)?.inc();

        let event = new_connection.event();

        let max_players: usize = config.general().max_players.into();

        if sessions.len() >= max_players {
            log::warn!(
                "Max players reached: {}. Disconnecting new connection form {}",
                max_players,
                event.connection.peer_addr()
            );
            event.connection.disconnect();
            return Ok(());
        }

        let server_sessions_entity = server_sessions_list.single()?;

        let session = GameServerSession::new(event.connection.clone());
        let session_id = session.id();
        let session_name = Name::new(format!("{:?}", session_id));
        let session_entity = commands
            .spawn((
                session_name,
                session,
                DespawnChildOf(server_sessions_entity),
            ))
            .id();
        sessions.insert(session_id, session_entity);
        Ok(())
    }

    fn disconnection_handler(
        disconnect: Trigger<DisconnectionEvent<GameServerNetworkConfig>>,
        mut commands: Commands,
        mut sessions: ResMut<ServerSessions>,
        metrics: Res<Metrics>,
        session_entities: Query<(Ref<character::Table>, Ref<Account>)>,
    ) -> Result<()> {
        let event = disconnect.event();

        let session_id = SessionId::from(event.connection.id());
        let session_entity = sessions.get(&session_id)?;
        let (character_table, account) = session_entities.get(session_entity)?;

        log::debug!(
            "Account: {} (uuid: {}) (ip: {}) disconnected",
            account.name(),
            account.id(),
            event.connection.peer_addr()
        );

        if let Ok(character_entity) = character_table.character() {
            commands.trigger_targets(CharacterSave, character_entity);
        }

        commands.entity(session_entity).despawn();
        sessions.remove(&session_id);
        metrics.gauge(GameNetworkMetric::ActiveSessions)?.dec();
        Ok(())
    }

    fn spawn_server_sessions_entity(mut commands: Commands) {
        commands.spawn((GameServerSessions, Name::new("GameServerSessions")));
    }
}
