use bevy::prelude::*;

mod request_manor_list;

pub(crate) struct ManorPlugin;
impl Plugin for ManorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(request_manor_list::RequestManorListPlugin);
    }
}
