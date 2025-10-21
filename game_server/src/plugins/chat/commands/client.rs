use bevy::prelude::*;
use chrono::Timelike;
use game_core::{
    chat::UserCommand,
    network::packets::server::{GameServerPacket, GameServerPackets, SystemMessage},
};
use map::{WorldMap, WorldMapQuery};
use spatial::GameVec3;
use system_messages::Id;

pub struct ClientCommandsPlugin;
impl Plugin for ClientCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UserCommand>()
            .add_observer(client_command_executed)
            .add_observer(handle_location_command)
            .add_observer(handle_time_command);
    }
}

fn client_command_executed(client_command: Trigger<UserCommand>, _commands: Commands) {
    let cmd = client_command.event();
    debug!(
        "{} command received from entity: {:?}",
        cmd,
        client_command.target()
    );
}

fn handle_location_command(
    client_command: Trigger<UserCommand>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    #[cfg(debug_assertions)] world_map_query: WorldMapQuery,
) -> Result<()> {
    let session_entity = client_command.target();

    if let UserCommand::Location = client_command.event()
        && let Ok(current_transform) = transforms.get(session_entity)
    {
        let current_game_loc = GameVec3::from(current_transform.translation);
        let location = format!(
            "{}, {}, {}",
            current_game_loc.x, current_game_loc.y, current_game_loc.z
        );

        commands.trigger_targets(
            GameServerPackets::from(vec![
                SystemMessage::new(Id::CurrentLocationS1, vec![location.into()]).into(),
                SystemMessage::new(
                    Id::CurrentLocationS1,
                    vec![current_transform.translation.into()],
                )
                .into(),
            ]),
            session_entity,
        );

        #[cfg(debug_assertions)]
        {
            let geodata = world_map_query.region_geodata_from_pos(current_transform.translation)?;
            let block_kind =
                geodata.block_kind(&WorldMap::vec3_to_geo(current_transform.translation));
            debug!("block in location: {:?}", block_kind);
        }
    }

    Ok(())
}
fn handle_time_command(client_command: Trigger<UserCommand>, mut commands: Commands) {
    let session_entity = client_command.target();

    if let UserCommand::Time = client_command.event() {
        let current_time = chrono::Local::now();
        let hour = current_time.hour();
        let minute = current_time.minute();

        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                Id::TheCurrentTimeIsS1S22,
                vec![hour.into(), minute.into()],
            )),
            session_entity,
        );
    }
}
