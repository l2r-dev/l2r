use crate::plugins::player_specific::next_intention::NextIntentionPlugin;
use bevy::app::{App, Plugin};

pub mod next_intention;

pub struct PlayerSpecificPlugin;

impl Plugin for PlayerSpecificPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NextIntentionPlugin);
    }
}
