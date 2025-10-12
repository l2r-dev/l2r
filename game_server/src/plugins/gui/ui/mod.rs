use bevy::prelude::*;
use bevy_egui::{EguiContext, PrimaryEguiContext};

pub mod camera_controller;
pub mod state;
mod windows;
pub use windows::*;

pub fn show_ui_system(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
        .cloned()
    else {
        return;
    };

    world.resource_scope::<state::UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}
