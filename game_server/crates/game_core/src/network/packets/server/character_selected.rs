use super::GameServerPacketCodes;
use crate::{character, object_id::ObjectId, stats::*};
use bevy::prelude::*;
use core::fmt;
use l2r_core::{
    model::{race::Race, session::SessionId},
    packets::{L2rServerPacket, ServerPacketBuffer},
};
use spatial::GameVec3;

#[derive(Clone, Reflect)]
pub struct CharacterSelected {
    object_id: ObjectId,
    name: String,
    session_id: SessionId,
    clan_id: u32,
    appearance: character::Appearance,
    race: Race,
    class_id: ClassId,
    position: Vec3,
    vitals: VitalsStats,
    progress_stats: ProgressStats,
    progress_level: ProgressLevelStats,
    primal_stats: PrimalStats,
    pvp: PvpStats,
}
impl fmt::Debug for CharacterSelected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} - {:?}", self.name, self.object_id)
    }
}
impl L2rServerPacket for CharacterSelected {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        let position = GameVec3::from(self.position);
        let current_hp = self.vitals.get(VitalsStat::Hp) as f64;
        let current_mp = self.vitals.get(VitalsStat::Mp) as f64;
        buffer.extend(GameServerPacketCodes::CHARACTER_SELECTED.to_le_bytes());
        buffer.str(self.name.as_str());
        buffer.u32(self.object_id.into());
        buffer.str(self.name.as_str());
        buffer.u32_from_usize(*self.session_id);
        buffer.u32(self.clan_id);
        buffer.u32(0); // unknown
        buffer.u32(self.appearance.gender.into());
        buffer.u32(self.race.into());
        buffer.u32(self.class_id.into());
        buffer.u32(1); // active ?
        buffer.extend(position.to_le_bytes());
        buffer.f64(current_hp);
        buffer.f64(current_mp);
        buffer.u32(self.progress_stats.sp());
        buffer.u64(self.progress_stats.exp());
        buffer.u32(self.progress_level.level().into());
        buffer.u32(self.pvp.karma);
        buffer.u32(self.pvp.pk_kills);
        buffer.u32(self.primal_stats.get(PrimalStat::INT));
        buffer.u32(self.primal_stats.get(PrimalStat::STR));
        buffer.u32(self.primal_stats.get(PrimalStat::CON));
        buffer.u32(self.primal_stats.get(PrimalStat::MEN));
        buffer.u32(self.primal_stats.get(PrimalStat::DEX));
        buffer.u32(self.primal_stats.get(PrimalStat::WIT));
        buffer.u32(0);
        buffer.u32(0);
        buffer.u32(self.class_id.into());
        buffer
    }
}
impl CharacterSelected {
    pub fn new(character: &character::Bundle, session_id: SessionId) -> Self {
        Self {
            name: character.name.to_string(),
            object_id: character.id,
            session_id,
            clan_id: 0,
            appearance: character.appearance,
            race: character.race,
            class_id: character.sub_class.class_id(),
            position: character.transform.translation,
            vitals: character.vitals_stats.clone(),
            progress_stats: character.progress_stats.clone(),
            progress_level: character.progress_level.clone(),
            primal_stats: character.primal_stats.clone(),
            pvp: character.pvp,
        }
    }
}
