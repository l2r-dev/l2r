use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    attack::Dead,
    character,
    network::{
        config::GameServerNetworkConfig,
        packets::{client::GameClientPacket, server::*},
        session::PacketReceiveParams,
    },
};
use spatial::Heading;
use system_messages::Id;

pub(crate) struct EnterWorldPlugin;
impl Plugin for EnterWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    characters: Query<character::Query>,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::EnterWorld = event.packet else {
        return Ok(());
    };

    let char_entity = receive_params.character(&event.connection.id())?;
    let selected_char = characters.get(char_entity)?;

    let mut enter_world_packets = GameServerPackets::from(vec![EtcStatusUpdate::new().into()]);
    enter_world_packets.push(ExBasicActionList.into());
    enter_world_packets.push(
        ExRotation::new(
            *selected_char.object_id,
            Heading::from(selected_char.transform.rotation),
        )
        .into(),
    );
    enter_world_packets.push(SystemMessage::new_empty(Id::WelcomeToTheWorldOfLineage2).into());
    enter_world_packets
        .push(ValidateLocation::new(*selected_char.object_id, *selected_char.transform).into());
    enter_world_packets.push(ActionFail.into());

    log::debug!("Enter world packets sent to entity {:?}", char_entity);
    commands.trigger_targets(enter_world_packets, char_entity);

    if selected_char.vitals_stats.dead() {
        commands.trigger_targets(Dead::new(char_entity), char_entity);
    }

    commands
        .entity(char_entity)
        .insert(game_core::encounters::EnteredWorld);

    Ok(())
}
