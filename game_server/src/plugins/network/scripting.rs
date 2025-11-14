use bevy::prelude::*;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::network::{
    broadcast::{BroadcastScope, ServerPacketBroadcast},
    config::GameServerNetworkConfig,
    packets::server::GameServerPacket,
    session::PacketReceiveParams,
};
use scripting::{
    bindings::{AppReflectAllocator, FunctionCallContext, InteropError, ReflectReference},
    core::{callback_labels, event::ScriptCallbackEvent, handler::event_handler},
    lua::LuaScriptingPlugin,
    prelude::{NamespaceBuilder, ScriptValue},
};
use std::any::TypeId;

pub struct GameServerPacketScriptingPlugin;
impl Plugin for GameServerPacketScriptingPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        NamespaceBuilder::<GameServerPacket>::new(world)
            .register("send", Self::script_send_packet)
            .register("broadcast", Self::script_broadcast_packet);
    }
}

impl GameServerPacketScriptingPlugin {
    fn script_send_packet(
        ctx: FunctionCallContext,
        script_value: ScriptValue,
    ) -> Result<(), InteropError> {
        let world = ctx.world()?;
        match script_value {
            ScriptValue::List(list) => {
                if list.len() != 2 {
                    return Err(InteropError::invariant("Expected a List [entity, packet]"));
                }

                let entity = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                let packet = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                if let (ScriptValue::Reference(entity_ref), ScriptValue::Reference(packet_ref)) =
                    (entity, packet)
                {
                    let entity_ref = entity_ref.downcast::<Entity>(world.clone())?;
                    let packet_ref = packet_ref.downcast::<GameServerPacket>(world.clone())?;
                    world.with_global_access(|world| {
                        world.trigger_targets(packet_ref, entity_ref);
                    })?;
                }
                Ok(())
            }
            _ => Err(InteropError::invariant(
                "Expected a List [entity, packet]".to_owned(),
            )),
        }
    }

    fn script_broadcast_packet(
        ctx: FunctionCallContext,
        script_value: ScriptValue,
    ) -> Result<(), InteropError> {
        let world = ctx.world()?;
        match script_value {
            ScriptValue::List(values) => {
                if values.len() != 3 {
                    return Err(InteropError::invariant(
                        "Expected a List [entity, packet, broadcastScope]",
                    ));
                }

                let entity = match &values[0] {
                    ScriptValue::Reference(entity_ref) => {
                        entity_ref.downcast::<Entity>(world.clone())?
                    }
                    _ => {
                        return Err(InteropError::type_mismatch(
                            TypeId::of::<Entity>(),
                            Some(TypeId::of::<ScriptValue>()),
                        ));
                    }
                };

                let packet = match &values[1] {
                    ScriptValue::Reference(packet_ref) => {
                        packet_ref.downcast::<GameServerPacket>(world.clone())?
                    }
                    _ => {
                        return Err(InteropError::type_mismatch(
                            TypeId::of::<GameServerPacket>(),
                            Some(TypeId::of::<ScriptValue>()),
                        ));
                    }
                };

                let scope = match &values[2] {
                    ScriptValue::Reference(scope_ref) => {
                        scope_ref.downcast::<BroadcastScope>(world.clone())?
                    }
                    _ => {
                        return Err(InteropError::string_type_mismatch(
                            "BroadcastScope".to_owned(),
                            None,
                        ));
                    }
                };

                world.with_global_access(|world| {
                    world.trigger_targets(ServerPacketBroadcast { packet, scope }, entity);
                })?;
                Ok(())
            }
            _ => Err(InteropError::unregistered_component_or_resource_type(
                "Expected a List [entity, packet, broadcastScope]".to_owned(),
            )),
        }
    }
}

pub struct ClientPacketScriptingPlugin;
impl Plugin for ClientPacketScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Self::send_to_scripts);

        app.add_systems(
            Update,
            event_handler::<OnPacketReceivedFunction, LuaScriptingPlugin>,
        );
    }
}

impl ClientPacketScriptingPlugin {
    fn send_to_scripts(
        receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
        receive_params: PacketReceiveParams,
        allocator: ResMut<AppReflectAllocator>,
        mut script_events: EventWriter<ScriptCallbackEvent>,
    ) -> Result<()> {
        let event = receive.event();
        let Ok(session_entity) = receive_params.session(&event.connection.id()) else {
            return Ok(());
        };
        let packet_entity = receive_params
            .character(&event.connection.id())
            .unwrap_or(session_entity);
        let packet_ref =
            ReflectReference::new_allocated(event.packet.clone(), &mut allocator.write());
        let entity_ref = ReflectReference::new_allocated(packet_entity, &mut allocator.write());
        script_events.write(ScriptCallbackEvent::new_for_all_contexts(
            OnPacketReceivedFunction,
            vec![entity_ref.into(), packet_ref.into()],
        ));
        Ok(())
    }
}

callback_labels!(
    OnPacketReceivedFunction => "on_packet_received",
);
