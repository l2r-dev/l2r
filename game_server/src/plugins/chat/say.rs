use super::{chat_logger::LogChatMessage, flood_protection::ChatCooldown};
use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    character::Character,
    chat::{CustomCommandExecuted, Kind},
    network::{
        broadcast::ServerPacketBroadcast,
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{CreatureSay, GameServerPacket, SystemMessage},
        },
        session::PacketReceiveParams,
    },
    object_id::ObjectId,
};
use system_messages::Id as SystemMessageId;

pub(crate) struct SayPlugin;
impl Plugin for SayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    mut chat_logs: EventWriter<LogChatMessage>,
    mut commands: Commands,
    mut characters: Query<
        (Entity, Ref<ObjectId>, Ref<Name>, Option<&mut ChatCooldown>),
        With<Character>,
    >,
    whisper_targets: Query<(Entity, Ref<Name>), With<Character>>,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::Say(ref packet) = event.packet {
        let character_entity = receive_params.character(&event.connection.id())?;
        if let Ok((_, char_oid, char_name, cooldown_opt)) = characters.get_mut(character_entity) {
            if packet.text.starts_with('.') {
                commands
                    .trigger_targets(CustomCommandExecuted(packet.text.clone()), character_entity);
                return Ok(());
            };

            // Check and handle cooldown for this specific chat type
            if let Some(mut cooldown) = cooldown_opt {
                if !cooldown.can_send(&packet.chat_type) {
                    let remaining = cooldown.remaining(&packet.chat_type);
                    let remaining_secs = remaining.as_secs_f32().ceil() as u32;
                    let remaining_minutes = remaining_secs.div_ceil(60); // Round up to nearest minute
                    commands.trigger_targets(
                        GameServerPacket::SystemMessage(SystemMessage::new(
                            SystemMessageId::ChatAvailableTimeS1Minute,
                            vec![system_messages::SmParam::Number(remaining_minutes)],
                        )),
                        character_entity,
                    );
                    return Ok(());
                }
                cooldown.start_cooldown(packet.chat_type);
            } else {
                let mut new_cooldown = ChatCooldown::default();
                new_cooldown.start_cooldown(packet.chat_type);
                commands.entity(character_entity).try_insert(new_cooldown);
            }

            let mut recievers = None;
            let mut target_name_for_log = None;
            if packet.chat_type == Kind::Whisper
                && let Some(target_name) = &packet.target
            {
                target_name_for_log = Some(target_name.clone());
                let target_name_lower = target_name.to_lowercase();
                let target_entities: Vec<Entity> = whisper_targets
                    .iter()
                    .filter(|(_, name)| name.to_string().to_lowercase() == target_name_lower)
                    .map(|(entity, _)| entity)
                    .collect();

                if target_entities.is_empty() {
                    commands.trigger_targets(
                        GameServerPacket::SystemMessage(SystemMessage::new(
                            SystemMessageId::S1CurrentlyOffline,
                            vec![system_messages::SmParam::Text(target_name.clone())],
                        )),
                        character_entity,
                    );
                    return Ok(());
                }
                recievers = Some(target_entities);

                commands.trigger_targets(
                    GameServerPacket::from(CreatureSay::new(
                        *char_oid,
                        format!("->{target_name}"),
                        vec![packet.text.clone()],
                        packet.chat_type,
                        None,
                    )),
                    character_entity,
                );
            }

            chat_logs.write(LogChatMessage {
                chat_type: packet.chat_type,
                sender: char_name.to_string(),
                target: target_name_for_log,
                message: packet.text.clone(),
            });

            let sending_packet = CreatureSay::new(
                *char_oid,
                char_name.to_string(),
                vec![packet.text.clone()],
                packet.chat_type,
                recievers.clone(),
            );
            let scope = sending_packet.broadcast_scope();
            commands.trigger_targets(
                ServerPacketBroadcast {
                    packet: sending_packet.into(),
                    scope,
                },
                character_entity,
            );
        }
    }
    Ok(())
}
