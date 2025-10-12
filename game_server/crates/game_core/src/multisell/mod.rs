use bevy::prelude::*;

pub mod admin_shop;
mod entry;
mod good;
mod id;

pub use entry::*;
pub use good::*;
pub use id::*;

pub struct MultisellComponentsPlugin;
impl Plugin for MultisellComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Id>()
            .register_type::<Entry>()
            .register_type::<Good>();
    }
}
