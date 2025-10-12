use bevy::prelude::*;
use game_core::{
    character::Character,
    encounters::KnownEntities,
    network::packets::server::{GameServerPacket, TeleportToLocation},
    teleport::{TeleportComponentsPlugin, TeleportInProgress},
};

mod destination;

pub struct TeleportPlugin;
impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TeleportComponentsPlugin)
            .add_plugins(destination::TeleportDestinationsPlugin);

        app.add_observer(teleport);
    }
}

fn teleport(
    teleport: Trigger<TeleportToLocation>,
    mut characters: Query<(Mut<Transform>, Mut<KnownEntities>), With<Character>>,
    mut commands: Commands,
) {
    let entity = teleport.target();
    if let Ok((mut transform, mut known_entities)) = characters.get_mut(entity) {
        commands.entity(entity).try_insert(TeleportInProgress);
        *transform = teleport.transform();
        known_entities.clear();
    }

    commands.trigger_targets(GameServerPacket::from(teleport.event().clone()), entity);
}
