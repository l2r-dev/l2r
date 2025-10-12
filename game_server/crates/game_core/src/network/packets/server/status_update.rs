use super::GameServerPacketCodes;
use crate::{object_id::ObjectId, stats::*};
use bevy::prelude::*;
use derive_more::{From, Into};
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::IntoPrimitive;
use strum::EnumIter;

#[repr(u32)]
#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, IntoPrimitive, PartialEq, Reflect)]
pub enum StatusUpdateKind {
    Level = 0x01,
    Exp = 0x02,
    Str = 0x03,
    Dex = 0x04,
    Con = 0x05,
    Int = 0x06,
    Wit = 0x07,
    Men = 0x08,
    CurHp = 0x09,
    MaxHp = 0x0a,
    CurMp = 0x0b,
    MaxMp = 0x0c,
    Sp = 0x0d,
    WeightCurrent = 0x0e,
    WeightLimit = 0x0f,
    PAtk = 0x11,
    AtkSpd = 0x12,
    PDef = 0x13,
    Evasion = 0x14,
    Accuracy = 0x15,
    Critical = 0x16,
    MAtk = 0x17,
    CastSpd = 0x18,
    MDef = 0x19,
    PvpFlag = 0x1a,
    Karma = 0x1b,
    CurCp = 0x21,
    MaxCp = 0x22,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct StatusUpdateField {
    kind: StatusUpdateKind,
    value: u32,
}

#[derive(Clone, Debug, Default, Deref, DerefMut, From, Into, PartialEq, Reflect)]
pub struct StatusUpdateFields(Vec<StatusUpdateField>);

#[derive(Clone, Debug, Reflect)]
pub struct StatusUpdate {
    object_id: ObjectId,
    fields: StatusUpdateFields,
}
impl StatusUpdate {
    pub fn new(object_id: ObjectId) -> Self {
        StatusUpdate {
            object_id,
            fields: StatusUpdateFields::default(),
        }
    }

    pub fn add(&mut self, kind: StatusUpdateKind, value: u32) {
        self.fields.push(StatusUpdateField { kind, value });
    }

    pub fn fields(&self) -> &StatusUpdateFields {
        &self.fields
    }
}

impl L2rServerPacket for StatusUpdate {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::STATUS_UPDATE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32_from_usize(self.fields.len());
        for field in self.fields.iter() {
            buffer.u32(field.kind.into());
            buffer.u32(field.value);
        }
        buffer
    }
}

impl From<(ObjectId, &VitalsStats)> for StatusUpdate {
    fn from((object_id, stats): (ObjectId, &VitalsStats)) -> Self {
        let mut status_update = StatusUpdate::new(object_id);

        let current_hp = stats.get(&VitalsStat::Hp) as u32;
        let current_mp = stats.get(&VitalsStat::Mp) as u32;
        let current_cp = stats.get(&VitalsStat::Cp) as u32;

        status_update.add(StatusUpdateKind::CurHp, current_hp);
        status_update.add(StatusUpdateKind::CurMp, current_mp);
        status_update.add(StatusUpdateKind::CurCp, current_cp);

        let max_hp = stats.get(&VitalsStat::MaxHp) as u32;
        let max_mp = stats.get(&VitalsStat::MaxMp) as u32;
        let max_cp = stats.get(&VitalsStat::MaxCp) as u32;

        status_update.add(StatusUpdateKind::MaxHp, max_hp);
        status_update.add(StatusUpdateKind::MaxMp, max_mp);
        status_update.add(StatusUpdateKind::MaxCp, max_cp);

        status_update
    }
}

impl From<&VitalsStat> for StatusUpdateKind {
    fn from(stat: &VitalsStat) -> Self {
        match stat {
            VitalsStat::Hp => StatusUpdateKind::CurHp,
            VitalsStat::Mp => StatusUpdateKind::CurMp,
            VitalsStat::Cp => StatusUpdateKind::CurCp,
            VitalsStat::MaxHp => StatusUpdateKind::MaxHp,
            VitalsStat::MaxMp => StatusUpdateKind::MaxMp,
            VitalsStat::MaxCp => StatusUpdateKind::MaxCp,
            _ => panic!("No corresponding StatusUpdateKind for {stat:?}"),
        }
    }
}
