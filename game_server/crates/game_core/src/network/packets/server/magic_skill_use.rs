use super::GameServerPacketCodes;
use crate::{object_id::ObjectId, skills::Skill};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::GameVec3;
use std::{fmt, time::Duration};

#[derive(Clone, Reflect)]
pub struct MagicSkillUse {
    user: ObjectId,
    origin_location: Vec3,
    target: ObjectId,
    target_location: Vec3,
    ground_location: Option<Vec3>,
    skill: Skill,
    hit_time: Duration,
    reuse_delay: Duration,
}
impl fmt::Debug for MagicSkillUse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> current: {}, target: {}",
            self.user, self.origin_location, self.target_location
        )
    }
}

impl L2rServerPacket for MagicSkillUse {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        let origin_location = GameVec3::from(self.origin_location);
        let target_location = GameVec3::from(self.target_location);

        buffer.extend(GameServerPacketCodes::MAGIC_SKILL_USE.to_le_bytes());
        buffer.u32(self.user.into());
        buffer.u32(self.target.into());
        buffer.u32(self.skill.display_id().into());
        buffer.u32(self.skill.level().into());
        buffer.u32(self.hit_time.as_millis() as u32);
        buffer.u32(self.reuse_delay.as_millis() as u32);
        buffer.extend(origin_location.to_le_bytes());
        buffer.u16(0); // isGroundTargetSkill?
        if let Some(ground_location) = self.ground_location {
            buffer.u16(1);
            buffer.extend(GameVec3::from(ground_location).to_le_bytes());
        } else {
            buffer.u16(0);
        }
        buffer.extend(target_location.to_le_bytes());
        buffer
    }
}
impl MagicSkillUse {
    pub fn new(
        user: ObjectId,
        origin_location: Vec3,
        target: ObjectId,
        target_location: Vec3,
        skill: Skill,
        hit_time: Duration,
        reuse_delay: Duration,
    ) -> Self {
        MagicSkillUse {
            user,
            origin_location,
            target,
            target_location,
            ground_location: None,
            skill,
            hit_time,
            reuse_delay,
        }
    }
}
