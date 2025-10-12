use bevy::prelude::*;

mod id;
mod param;

pub use id::*;
pub use param::*;

pub struct SystemMessagesPlugin;
impl Plugin for SystemMessagesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmParam>();
    }
}
