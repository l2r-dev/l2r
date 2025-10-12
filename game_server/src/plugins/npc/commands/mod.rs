use bevy::{log, prelude::*};
use game_core::npc::NpcCommandVariants;
use sea_orm::Iterable;

mod chat;
mod tp;

pub struct NpcCommandsPlugin;
impl Plugin for NpcCommandsPlugin {
    fn build(&self, app: &mut App) {
        for command in NpcCommandVariants::iter() {
            match command {
                NpcCommandVariants::Tp => {
                    app.add_observer(tp::handle);
                }
                NpcCommandVariants::Chat => {
                    app.add_observer(chat::handle);
                }
                _ => {
                    log::trace!("Npc command {:?} is not implemented yet", command);
                }
            };
        }
    }
}
