use super::GameServerPacketCodes;
use crate::{
    abnormal_effects::{AbnormalEffect, AbnormalEffects, AbnormalEffectsTimers},
    skills::{Id, Skill},
};
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use std::fmt;

#[derive(Clone, Reflect)]
pub struct AbnormalStatusUpdate {
    effects: Vec<AbnormalEffect>,
    durations: Vec<i32>, // -1 for infinite, otherwise remaining seconds
}

impl fmt::Debug for AbnormalStatusUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:?}> AbnormalStatusUpdate [effects_count: {}]",
            GameServerPacketCodes::ABNORMAL_STATUS_UPDATE,
            self.effects.len(),
        )
    }
}

impl L2rServerPacket for AbnormalStatusUpdate {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::ABNORMAL_STATUS_UPDATE.to_le_bytes());

        buffer.u16(self.effects.len() as u16);
        for (i, effect) in self.effects.iter().enumerate() {
            let skill: &Skill = effect.as_ref();
            buffer.u32(skill.display_id().into());
            buffer.u16(skill.level().into());
            buffer.i32(self.durations.get(i).copied().unwrap_or(-1));
        }

        buffer
    }
}

impl AbnormalStatusUpdate {
    pub fn new(effects: &AbnormalEffects, timers: &AbnormalEffectsTimers) -> Self {
        let mut effect_list = Vec::new();
        let mut duration_list = Vec::new();

        for effect in effects.active_effects() {
            let skill_id: Id = effect.skill().id();
            let remaining_time = timers
                .get_timer(skill_id)
                .and_then(|timer_data| timer_data.timer())
                .map(|timer| timer.remaining().as_secs() as i32)
                .unwrap_or(-1);

            effect_list.push(*effect);
            duration_list.push(remaining_time);
        }

        Self {
            effects: effect_list,
            durations: duration_list,
        }
    }
}
