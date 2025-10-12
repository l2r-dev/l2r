use bevy::{app::AppExit, log, prelude::*};
use game_core::character::{Character, CharacterSave};
use state::GameServerStateSystems;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub struct ShutdownPlugin;

impl Plugin for ShutdownPlugin {
    fn build(&self, app: &mut App) {
        let shutdown_requested = Arc::new(AtomicBool::new(false));

        let shutdown_state = ShutdownState {
            shutdown_requested: shutdown_requested.clone(),
            total_chars: 0,
            saved_chars: 0,
            shutdown_start: None,
        };

        let shutdown_requested_clone = shutdown_requested.clone();
        // Only set the handler if it hasn't been set already
        if let Err(e) = ctrlc::set_handler(move || {
            if shutdown_requested_clone.load(Ordering::Acquire) {
                log::warn!("Force shutdown requested!");
                std::process::exit(1);
            } else {
                log::info!("Shutdown signal received, saving everything...");
                shutdown_requested_clone.store(true, Ordering::Release);
            }
        }) {
            match e {
                ctrlc::Error::MultipleHandlers => {
                    log::debug!("Ctrl-C handler already set, skipping...");
                }
                _ => {
                    log::error!("Error setting Ctrl-C handler: {:?}", e);
                }
            }
        }

        app.insert_resource(shutdown_state);
        app.add_systems(Update, check_shutdown_signal);
    }
}

#[derive(Resource)]
pub struct ShutdownState {
    pub shutdown_requested: Arc<AtomicBool>,
    pub total_chars: usize,
    pub saved_chars: usize,
    shutdown_start: Option<Instant>,
}

fn check_shutdown_signal(
    mut shutdown_state: ResMut<ShutdownState>,
    mut commands: Commands,
    characters: Query<Entity, With<Character>>,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameServerStateSystems>>,
) {
    if shutdown_state.shutdown_requested.load(Ordering::Acquire) {
        // Initialize shutdown if this is the first time we detect it
        if shutdown_state.shutdown_start.is_none() {
            let char_count = characters.iter().count();
            shutdown_state.total_chars = char_count;
            shutdown_state.shutdown_start = Some(Instant::now());

            log::info!("Graceful shutdown initiated. Saving world state...");
            next_state.set(GameServerStateSystems::Shutdown);
            log::info!("Saving {} active character(s)...", char_count);
            for entity in characters.iter() {
                commands.trigger_targets(CharacterSave, entity);
            }
            log::info!("Waiting up to 10 seconds for pending operations...");
        }

        let elapsed = shutdown_state.shutdown_start.unwrap().elapsed();
        let saved = shutdown_state.saved_chars;
        let total = shutdown_state.total_chars;

        // Check if all characters are saved or timeout exceeded
        if saved >= total {
            log::info!(
                "Character save completed ({}/{} in {:.2}s)",
                saved,
                total,
                elapsed.as_secs_f32()
            );
            log::info!("All pending operations completed. Shutting down gracefully...");
            exit.write(AppExit::Success);
        } else if elapsed >= Duration::from_secs(10) {
            log::warn!("Graceful shutdown timeout reached after 10 seconds!");
            log::warn!(
                "{}/{} character(s) saved. Forcing shutdown...",
                saved,
                total
            );
            exit.write(AppExit::Success);
        }
    }
}
