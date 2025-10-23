use super::GameServerPacketCodes;
use crate::{attack::HitInfo, object_id::ObjectId, stats::ShieldResult};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::fmt;

#[repr(u8)]
#[derive(Debug, Reflect)]
pub enum HitFlag {
    UseSs = 0x10,
    Crit = 0x20,
    Shield = 0x40,
    Miss = 0x80,
}

#[derive(Clone, Debug, Reflect)]
pub struct Hit {
    damage: u32,
    flags: u8,
    target: ObjectId,
}

impl Hit {
    pub fn new(target: ObjectId, hit_info: HitInfo) -> Self {
        let mut flags: u8 = 0;

        if let Some(ss_grade) = hit_info.ss_grade {
            flags |= HitFlag::UseSs as u8 | ss_grade as u8;
        }

        if hit_info.crit {
            flags |= HitFlag::Crit as u8;
        }

        if matches!(
            hit_info.shield,
            ShieldResult::Succeed | ShieldResult::PerfectBlock
        ) {
            flags |= HitFlag::Shield as u8;
        }

        if hit_info.miss {
            flags |= HitFlag::Miss as u8;
        }

        Self {
            damage: hit_info.damage as u32,
            flags,
            target,
        }
    }

    pub fn to_le_bytes(&self) -> [u8; 9] {
        let mut bytes = [0u8; 9];
        bytes[0..4].copy_from_slice(&self.target.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.damage.to_le_bytes());
        bytes[8] = self.flags;
        bytes
    }
}

#[derive(Clone, Reflect)]
pub struct Attack {
    attacker: ObjectId,
    attacker_loc: Vec3,
    target_loc: Vec3,
    hits: Vec<Hit>,
}

impl Attack {
    pub fn new(attacker: ObjectId, attacker_loc: Vec3, target_loc: Vec3) -> Self {
        Self {
            attacker,
            attacker_loc,
            target_loc,
            hits: Vec::with_capacity(2), // Most attacks have 1 or 2 hits
        }
    }
}
impl Attack {
    #[allow(clippy::too_many_arguments)]
    pub fn add_hit(&mut self, target: ObjectId, hit_info: HitInfo) {
        self.hits.push(Hit::new(target, hit_info));
    }
}

impl fmt::Debug for Attack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attack {{ attacker: {:?}, attacker_loc: {:?}, target_loc: {:?}, hits: {:?} }}",
            self.attacker, self.attacker_loc, self.target_loc, self.hits
        )
    }
}

impl L2rServerPacket for Attack {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        let attacker_loc = GameVec3::from(self.attacker_loc);
        let target_loc = GameVec3::from(self.target_loc);

        buffer.extend(GameServerPacketCodes::ATTACK.to_le_bytes());
        buffer.u32(self.attacker.into());

        buffer.extend(self.hits.first().unwrap().to_le_bytes());

        buffer.extend(attacker_loc.to_le_bytes());
        buffer.u16_from_usize(self.hits.len() - 1);

        self.hits.iter().skip(1).for_each(|hit| {
            buffer.extend(hit.to_le_bytes());
        });

        buffer.extend(target_loc.to_le_bytes());

        buffer
    }
}
