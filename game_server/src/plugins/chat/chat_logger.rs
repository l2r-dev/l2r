use bevy::prelude::*;
use chrono::{DateTime, Local};
use game_core::chat::Kind;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

const LOG_DIR: &str = "logs";
const FLUSH_INTERVAL_MS: u64 = 100;

pub struct ChatLoggerPlugin;

impl Plugin for ChatLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatLogs>()
            .add_event::<LogChatMessage>()
            .add_systems(Update, (log_chat_message, flush_chat_logs));
    }
}

/// Bevy resource that buffers chat messages before writing to disk
#[derive(Resource)]
pub struct ChatLogs {
    buffer: Vec<ChatLogEntry>,
    timer: Timer,
}

impl Default for ChatLogs {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
            timer: Timer::from_seconds(FLUSH_INTERVAL_MS as f32 / 1000.0, TimerMode::Repeating),
        }
    }
}

#[derive(Clone)]
struct ChatLogEntry {
    timestamp: DateTime<Local>,
    chat_type: Kind,
    sender: String,
    target: Option<String>,
    message: String,
}

impl ChatLogEntry {
    fn format(&self) -> String {
        let time = self.timestamp.format("%H:%M:%S");
        match &self.target {
            Some(target) => format!(
                "[{}] [{}] {} -> {}: {}\n",
                time, self.chat_type, self.sender, target, self.message
            ),
            None => format!(
                "[{}] [{}] {}: {}\n",
                time, self.chat_type, self.sender, self.message
            ),
        }
    }
}

impl ChatLogs {
    pub fn add_message(
        &mut self,
        chat_type: Kind,
        sender: String,
        target: Option<String>,
        message: String,
    ) {
        self.buffer.push(ChatLogEntry {
            timestamp: Local::now(),
            chat_type,
            sender,
            target,
            message,
        });
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Ensure log directory exists
        fs::create_dir_all(LOG_DIR)?;

        // Get current date for filename
        let now = Local::now();
        let date_str = now.format("%d_%m_%Y");
        let log_file_path = PathBuf::from(LOG_DIR).join(format!("chat_{}.log", date_str));

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;

        // Write all buffered messages
        for entry in &self.buffer {
            file.write_all(entry.format().as_bytes())?;
        }

        // Clear buffer after successful write
        self.buffer.clear();

        Ok(())
    }
}

/// Event triggered when a chat message should be logged
#[derive(Event)]
pub struct LogChatMessage {
    pub chat_type: Kind,
    pub sender: String,
    pub target: Option<String>,
    pub message: String,
}

/// System that captures chat messages and adds them to the buffer
fn log_chat_message(mut chat_logs: ResMut<ChatLogs>, mut events: EventReader<LogChatMessage>) {
    for event in events.read() {
        chat_logs.add_message(
            event.chat_type,
            event.sender.clone(),
            event.target.clone(),
            event.message.clone(),
        );
    }
}

/// System that periodically flushes the buffer to disk
fn flush_chat_logs(mut chat_logs: ResMut<ChatLogs>, time: Res<Time>) {
    chat_logs.timer.tick(time.delta());

    if chat_logs.timer.just_finished()
        && let Err(e) = chat_logs.flush()
    {
        error!("Failed to flush chat logs: {}", e);
    }
}
