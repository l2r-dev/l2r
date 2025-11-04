use bevy::prelude::*;
use bevy_ecs::query::QueryData;
use game_core::{object_id::ObjectId, stats::VitalsStats};

#[derive(QueryData)]
pub struct DoorQuery<'a> {
    pub object_id: Ref<'a, ObjectId>,
    pub zone: Ref<'a, map::Zone>,
    pub vitals: Ref<'a, VitalsStats>,
    pub status: Ref<'a, map::DoorStatus>,
    pub mesh_info: Ref<'a, map::MeshInfo>,
}
