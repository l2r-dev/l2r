use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    abnormal_effects::{AbnormalEffects, AbnormalEffectsTimers},
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    skills::Skill,
};

pub(crate) struct RequestDispelPlugin;
impl Plugin for RequestDispelPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_request_dispel_packet);
    }
}

fn handle_request_dispel_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    recieve_params: PacketReceiveParams,
    mut abnormals: Query<(Mut<AbnormalEffects>, Mut<AbnormalEffectsTimers>)>,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestDispel(ref packet) = event.packet else {
        return Ok(());
    };
    let character_entity = recieve_params.character(&event.connection.id())?;
    let (mut effects, mut timers) = abnormals.get_mut(character_entity)?;
    effects
        .buffs_mut()
        .retain(|effect| Skill::id(effect.as_ref()) != packet.skill_id);
    timers.remove(packet.skill_id);
    Ok(())
}
