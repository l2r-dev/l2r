use bevy::{log, prelude::*};
pub use kind::Kind;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::Display;

mod kind;

pub struct ChatComponentsPlugin;
impl Plugin for ChatComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CustomCommandExecuted>()
            .add_event::<UserCommand>();
    }
}

#[repr(i32)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Eq,
    IntoPrimitive,
    PartialEq,
    TryFromPrimitive,
    Event,
    Reflect,
)]
pub enum UserCommand {
    #[default]
    Unknown = -1,
    Location = 0,
    Unstuck = 52,
    Time = 77,
}
impl UserCommand {
    pub fn new(id: i32) -> Self {
        Self::try_from(id).unwrap_or_default()
    }
}

impl From<&[u8]> for UserCommand {
    fn from(slice: &[u8]) -> Self {
        let command_id = i32::from_le_bytes(slice.try_into().unwrap_or_default());
        let command = Self::new(command_id);
        if command == UserCommand::Unknown {
            log::warn!("Unknown user command: {}", command_id);
        }
        command
    }
}

#[derive(Debug, Event)]
pub struct CustomCommandExecuted(pub String);
