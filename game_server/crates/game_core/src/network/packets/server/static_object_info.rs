use crate::{
    network::packets::server::GameServerPacketCodes,
    object_id::ObjectId,
    stats::{VitalsStat, VitalsStats},
};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use map::{DoorKind, DoorStatus, MeshInfo};

/// StaticObjectInfo packet
/// Sends information about static objects (doors, static meshes)
#[derive(Clone, Debug, Reflect)]
pub struct StaticObjectInfo {
    static_object_id: u32,
    object_id: ObjectId,
    object_type: StaticObjectType,
    is_targetable: bool,
    mesh_info: MeshInfo,
    is_closed: bool,
    is_enemy: bool,
    current_hp: u32,
    max_hp: u32,
    show_hp: bool,
    damage_grade: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum StaticObjectType {
    StaticObject,
    Door,
}

impl StaticObjectInfo {
    /// Create StaticObjectInfo for a door
    pub fn door(
        object_id: ObjectId,
        door_info: &DoorKind,
        door_vitals: &VitalsStats,
        status: DoorStatus,
        mesh_info: MeshInfo,
        is_enemy: bool,
    ) -> Self {
        let current_hp = door_vitals.get(VitalsStat::Hp) as u32;
        let damage_grade = Self::calculate_damage_grade(current_hp, door_info.max_hp);

        Self {
            static_object_id: door_info.id.into(),
            object_id,
            object_type: StaticObjectType::Door,
            is_targetable: door_info.targetable,
            mesh_info: mesh_info,
            is_closed: status.into(),
            is_enemy,
            current_hp,
            max_hp: door_info.max_hp,
            show_hp: door_info.show_hp,
            damage_grade,
        }
    }

    /// Create StaticObjectInfo for a static object
    pub fn static_object(static_object_id: u32, object_id: ObjectId, mesh_info: MeshInfo) -> Self {
        Self {
            static_object_id,
            object_id,
            object_type: StaticObjectType::StaticObject,
            is_targetable: true,
            mesh_info,
            is_closed: false,
            is_enemy: false,
            current_hp: 0,
            max_hp: 0,
            show_hp: false,
            damage_grade: 0,
        }
    }

    /// Calculate damage grade (0-6) based on current HP
    fn calculate_damage_grade(current_hp: u32, max_hp: u32) -> u32 {
        if max_hp == 0 {
            return 0;
        }

        let hp_ratio = current_hp as f32 / max_hp as f32;
        let dmg = 6 - (hp_ratio * 6.0).ceil() as i32;

        dmg.clamp(0, 6) as u32
    }
}

impl L2rServerPacket for StaticObjectInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::STATIC_OBJECT.to_le_bytes());
        buffer.i32(self.static_object_id as i32);
        buffer.i32(self.object_id.into());
        buffer.i32(self.object_type as i32);
        buffer.u32(self.is_targetable as u32);
        buffer.i32(self.mesh_info as i32);
        buffer.u32(self.is_closed as u32);
        buffer.u32(self.is_enemy as u32);
        buffer.i32(self.current_hp as i32);
        buffer.i32(self.max_hp as i32);
        buffer.u32(self.show_hp as u32);
        buffer.i32(self.damage_grade as i32);
        buffer
    }
}
