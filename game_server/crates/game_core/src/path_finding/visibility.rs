use bevy::prelude::*;

pub(crate) struct GeoVisibilityComponentsPlugin;

impl Plugin for GeoVisibilityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<VisibilityCheckRequest>()
            .add_event::<VisibilityCheckResult>();
    }
}

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct VisibilityCheckRequest {
    pub entity: Entity,
    pub start: Vec3,
    pub target: Vec3,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct VisibilityCheckResult {
    pub start: Vec3,
    pub target: Vec3,
    pub is_visible: bool,
}
