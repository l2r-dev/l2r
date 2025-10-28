use super::GameServerPacketCodes;
use crate::skills::SkillList;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

impl L2rServerPacket for SkillList {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::SKILL_LIST.to_le_bytes());
        buffer.u32_from_usize(self.len());

        for skill in self.values() {
            buffer.u32_from_bool(skill.passive());
            buffer.u32(skill.level().into());
            buffer.u32(skill.display_id().into());
            buffer.bool(skill.disabled());
            buffer.bool(skill.enchanted());
        }
        buffer
    }
}
