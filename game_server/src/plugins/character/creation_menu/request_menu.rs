use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    network::{
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{GameServerPacket, ResponseCharCreateMenu},
        },
        session::PacketReceiveParams,
    },
    stats::StatsTableQuery,
};

pub(crate) struct RequestCharCreateMenuPlugin;
impl Plugin for RequestCharCreateMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    stats_table: StatsTableQuery,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::RequestCharCreateMenu = event.packet {
        let session_entity = receive_params.session(&event.connection.id())?;
        let race_stats = stats_table.race_stats();
        commands.trigger_targets(
            GameServerPacket::from(ResponseCharCreateMenu::new(race_stats)),
            session_entity,
        );
    }
    Ok(())
}
