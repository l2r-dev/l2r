use bevy::prelude::*;

mod action_packet;
mod pickup;
mod request_action_use;
mod request_canclel_target;
mod request_restart;
mod request_restart_point;
mod request_show_map;
mod sit_stand;
mod social;
mod target;

pub struct UseActionPlugin;
impl Plugin for UseActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(action_packet::ActionPacketPlugin)
            .add_plugins(request_action_use::RequestActionUsePlugin)
            .add_plugins(request_canclel_target::RequestCancelTargetPlugin)
            .add_plugins(request_restart_point::RequestRestartPointPlugin)
            .add_plugins(request_restart::RequestRestartPlugin)
            .add_plugins(request_show_map::RequestShowMapPlugin)
            .add_plugins(target::TargetPlugin)
            .add_plugins(sit_stand::SitStandPlugin)
            .add_plugins(pickup::PickupPlugin);

        app.add_plugins(social::SeeDebugPlugin);
    }
}
