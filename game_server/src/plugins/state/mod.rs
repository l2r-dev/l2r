use bevy::{log, prelude::*};
use game_core::{
    chat,
    network::{
        broadcast::{BroadcastScope, ServerPacketBroadcast},
        packets::server::{CreatureSay, GameServerPacket},
    },
    object_id::ObjectId,
};
use state::{GameServerStatePlugin, GameServerStateSystems, TogglePause};

mod loading;

pub use loading::*;

pub struct GameStateProcessPlugin;
impl Plugin for GameStateProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameServerStatePlugin);
        app.add_plugins(LoadingProcessPlugin);

        app.add_systems(Update, pause_message.in_set(GameServerStateSystems::Pause));

        app.add_observer(toggle_pause_handler);
    }
}

fn toggle_pause_handler(
    trigger: Trigger<TogglePause>,
    state: Res<State<GameServerStateSystems>>,
    mut next_state: ResMut<NextState<GameServerStateSystems>>,
) {
    log::debug!("[{}] Toggle pause", trigger.target());
    match state.get() {
        GameServerStateSystems::ForcePause => next_state.set(GameServerStateSystems::Run),
        GameServerStateSystems::Run => next_state.set(GameServerStateSystems::ForcePause),
        _ => (),
    }
}

fn pause_message(mut commands: Commands, time: Res<Time>, mut last_time: Local<f32>) {
    let time_spent = time.elapsed_secs() - *last_time;
    if (time_spent) >= 5.0 {
        *last_time = time.elapsed_secs();
        let pause_message = "Paused!".to_string();
        log::warn!(pause_message);
        commands.trigger(ServerPacketBroadcast {
            packet: GameServerPacket::from(CreatureSay::new(
                ObjectId::default(),
                "Server".to_string(),
                vec![pause_message],
                chat::Kind::ScreenAnnounce,
                None,
            )),
            scope: BroadcastScope::All,
        });
    }
}
