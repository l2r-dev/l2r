use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    action::model::*,
    animation::Animation,
    attack::Dead,
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    player_specific::next_intention::NextIntention,
};

pub(crate) struct RequestActionUsePlugin;
impl Plugin for RequestActionUsePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut commands: Commands,
    characters: Query<Has<Animation>, Without<Dead>>,
) -> Result<()> {
    let event = receive.event();

    if let GameClientPacket::RequestActionUse(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;

        let Ok(has_animation) = characters.get(character_entity) else {
            return Ok(());
        };

        match packet.action_id {
            ActionId::Core(action) => {
                if has_animation {
                    commands
                        .entity(character_entity)
                        .try_insert(NextIntention::CoreAction(action));
                } else {
                    commands.trigger_targets(action, character_entity);
                }
            }
            ActionId::Servitor(action) => {
                commands.trigger_targets(action, character_entity);
            }
            ActionId::Special(action) => {
                if has_animation {
                    commands
                        .entity(character_entity)
                        .try_insert(NextIntention::SpecialAction(action));
                } else {
                    commands.trigger_targets(action, character_entity);
                }
            }
        }
    }

    Ok(())
}
