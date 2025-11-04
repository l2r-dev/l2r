use crate::{
    network::packets::server::GameServerPacketCodes,
    object_id::ObjectId,
    stats::{VitalsStat, VitalsStats},
};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use map::{DoorId, DoorKind, DoorStatus};

#[derive(Clone, Debug, Reflect)]
pub struct DoorStatusUpdate {
    object_id: ObjectId,
    is_closed: bool,
    damage_grade: u32,
    is_enemy: bool,
    door_id: DoorId,
    current_hp: u32,
    max_hp: u32,
}

impl DoorStatusUpdate {
    pub fn new(
        door_info: &DoorKind,
        object_id: ObjectId,
        vitals: &VitalsStats,
        status: DoorStatus,
        is_enemy: bool,
    ) -> Self {
        let current_hp = vitals.get(VitalsStat::Hp) as u32;
        let damage_grade = Self::calculate_damage_grade(current_hp, door_info.max_hp);

        Self {
            object_id,
            is_closed: status.into(),
            damage_grade,
            is_enemy,
            door_id: door_info.id,
            current_hp,
            max_hp: door_info.max_hp,
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

impl L2rServerPacket for DoorStatusUpdate {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::DOOR_STATUS_UPDATE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32_from_bool(self.is_closed);
        buffer.u32(self.damage_grade);
        buffer.u32_from_bool(self.is_enemy);
        buffer.u32(self.door_id.into());
        buffer.u32(self.current_hp);
        buffer.u32(self.max_hp);
        buffer
    }
}

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct BroadcastDoorStatusUpdate;
