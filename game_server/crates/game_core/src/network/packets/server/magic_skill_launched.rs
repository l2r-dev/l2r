use super::GameServerPacketCodes;
use crate::{object_id::ObjectId, skills::Skill};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct MagicSkillLaunched {
    object_id: ObjectId,
    skill: Skill,
    targets: Vec<ObjectId>,
}
impl fmt::Debug for MagicSkillLaunched {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> skill: {:?}, targets: {:?}",
            self.object_id, self.skill, self.targets
        )
    }
}

impl L2rServerPacket for MagicSkillLaunched {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::MAGIC_SKILL_LAUNCHED.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.skill.display_id().into());
        buffer.u32(self.skill.level().into());
        buffer.u32_from_usize(self.targets.len());
        for target in self.targets {
            buffer.u32(target.into());
        }
        buffer
    }
}
impl MagicSkillLaunched {
    pub fn new(object_id: ObjectId, skill: Skill, targets: Vec<ObjectId>) -> Self {
        MagicSkillLaunched {
            object_id,
            skill,
            targets,
        }
    }
}
