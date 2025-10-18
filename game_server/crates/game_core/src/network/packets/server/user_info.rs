use super::{GameServerPacketCodes, GameServerPackets, ex_br_extra_user_info::ExBrExtraUserInfo};
use crate::{character, items::PaperDoll, object_id::ObjectId, stats::*};
use bevy::prelude::*;
use core::fmt;
use l2r_core::{
    model::{base_class::BaseClass, race::Race},
    packets::{L2rServerPacket, ServerPacketBuffer},
};
use spatial::GameVec3;

#[derive(Event, Reflect)]
pub struct SendUserInfo;

#[derive(Event, Reflect)]
pub struct UserInfoUpdated(pub Entity);

#[derive(Clone, Reflect)]
pub struct UserInfo {
    pub name: String,
    pub title: String,
    pub race: Race,
    pub base_class: BaseClass,
    pub class_id: ClassId,
    pub object_id: ObjectId,
    pub progress_stats: ProgressStats,
    pub progress_level: ProgressLevelStats,
    pub primal_stats: PrimalStats,
    pub attack_stats: AttackStats,
    pub defence_stats: DefenceStats,
    pub critical_stats: CriticalStats,
    pub vitals_stats: VitalsStats,
    pub pvp_stats: PvpStats,
    pub movable: Movable,
    pub base_speed: MovementStats,
    pub appearance: character::Appearance,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub position: GameVec3,
    pub paper_doll: PaperDoll,
}
impl fmt::Debug for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:?}> {:?} - {:?} lvl {:?}",
            self.object_id,
            self.name,
            self.progress_level.level(),
            self.position
        )
    }
}
impl UserInfo {
    pub fn from_bundle(bundle: character::Bundle, base_speed: MovementStats) -> Self {
        let collision_radius = bundle.collider.radius();
        let collision_height = bundle.collider.height();

        Self {
            name: bundle.name.to_string(),
            title: bundle.title.to_string(),
            race: bundle.race,
            base_class: bundle.base_class,
            class_id: bundle.sub_class.class_id(),
            object_id: bundle.id,
            progress_stats: bundle.progress_stats,
            progress_level: bundle.progress_level,
            primal_stats: bundle.primal_stats,
            attack_stats: bundle.attack_stats,
            defence_stats: bundle.defence_stats,
            critical_stats: bundle.critical_stats,
            vitals_stats: bundle.vitals_stats,
            pvp_stats: bundle.pvp,
            movable: bundle.movable,
            base_speed,
            appearance: bundle.appearance,
            collision_radius,
            collision_height,
            position: GameVec3::from(bundle.transform.translation),
            paper_doll: bundle.paper_doll,
        }
    }

    pub fn from_query<'a, 'b>(
        character: &'a character::QueryItem<'a, 'b>,
        base_speed: MovementStats,
    ) -> Self {
        let collision_radius = character.collider.radius();
        let collision_height = character.collider.height();

        Self {
            name: character.name.to_string(),
            title: character.title.to_string(),
            race: *character.race,
            base_class: *character.base_class,
            class_id: character.sub_class.class_id(),
            object_id: *character.object_id,
            progress_stats: character.progress_stats.clone(),
            progress_level: character.progress_level.clone(),
            primal_stats: character.primal_stats.clone(),
            attack_stats: character.attack_stats.clone(),
            defence_stats: character.defence_stats.clone(),
            critical_stats: character.critical_stats.clone(),
            vitals_stats: character.vitals_stats.clone(),
            pvp_stats: *character.pvp_stats,
            movable: character.movable.clone(),
            base_speed,
            appearance: *character.appearance,
            collision_radius,
            collision_height,
            position: GameVec3::from(character.transform.translation),
            paper_doll: character.paperdoll.clone(),
        }
    }

    pub fn with_extra(self) -> GameServerPackets {
        let ex_br_extra_user_info = ExBrExtraUserInfo::new(self.object_id, 0, 0);
        GameServerPackets(vec![self.into(), ex_br_extra_user_info.into()])
    }
}
impl L2rServerPacket for UserInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();

        let p_atk = self.attack_stats.typed::<PAtk>(AttackStat::PAtk);
        let p_atk_spd = self.attack_stats.typed::<PAtkSpd>(AttackStat::PAtkSpd);
        let p_atk_spd_multiplier = p_atk_spd.get_attack_speed_multiplier();
        let accuracy = self.attack_stats.get(AttackStat::Accuracy) as u32;
        let m_atk = self.attack_stats.typed::<MAtk>(AttackStat::MAtk);
        let cast_spd = self.attack_stats.get(AttackStat::CastSpd) as u32;
        let physical_critical_rate = self.critical_stats.get(CriticalStat::CriticalRate) as u32;
        let p_def = self.defence_stats.get(DefenceStat::PDef) as u32;
        let m_def = self.defence_stats.get(DefenceStat::MDef) as u32;
        let evasion = self.defence_stats.get(DefenceStat::Evasion) as u32;

        let max_hp = self.vitals_stats.get(VitalsStat::MaxHp) as u32;
        let current_hp = self.vitals_stats.get(VitalsStat::Hp) as u32;
        let max_mp = self.vitals_stats.get(VitalsStat::MaxMp) as u32;
        let current_mp = self.vitals_stats.get(VitalsStat::Mp) as u32;
        let max_cp = self.vitals_stats.get(VitalsStat::MaxCp) as u32;
        let current_cp = self.vitals_stats.get(VitalsStat::Cp) as u32;
        let char_level = self.progress_level.level();

        buffer.extend(GameServerPacketCodes::USER_INFO.to_le_bytes());
        buffer.extend(self.position.to_le_bytes());
        buffer.u32(0); // stub for vehicle id
        buffer.u32(self.object_id.into());
        buffer.str(&self.name);
        buffer.u32(self.race.into());
        buffer.u32(self.appearance.gender.into());
        buffer.u32(self.class_id.into());
        buffer.u32(char_level.into());
        buffer.u64(self.progress_stats.exp());
        buffer.f64(self.progress_stats.exp_percent(char_level));
        buffer.u32(self.primal_stats.get(PrimalStat::STR));
        buffer.u32(self.primal_stats.get(PrimalStat::DEX));
        buffer.u32(self.primal_stats.get(PrimalStat::CON));
        buffer.u32(self.primal_stats.get(PrimalStat::INT));
        buffer.u32(self.primal_stats.get(PrimalStat::WIT));
        buffer.u32(self.primal_stats.get(PrimalStat::MEN));
        buffer.u32(max_hp);
        buffer.u32(current_hp);
        buffer.u32(max_mp);
        buffer.u32(current_mp);
        buffer.u32(self.progress_stats.sp());
        buffer.u32(0); // current load
        buffer.u32(17); // max load
        buffer.u32(40); // active weapon item

        // entity ids
        for slot_item in self.paper_doll.user_info_iter() {
            buffer.extend(
                slot_item
                    .unique_item()
                    .map(|u| u.object_id())
                    .unwrap_or(0.into())
                    .to_le_bytes(),
            );
        }
        // item ids
        for slot_item in self.paper_doll.user_info_iter() {
            buffer.extend(
                slot_item
                    .unique_item()
                    .map(|u| u.item().id())
                    .unwrap_or(0.into())
                    .to_le_bytes(),
            );
        }
        // augumented ids
        for slot_item in self.paper_doll.user_info_iter() {
            buffer.extend(
                slot_item
                    .unique_item()
                    .map(|u| u.item().id())
                    .unwrap_or(0.into())
                    .to_le_bytes(),
            );
        }
        buffer.u32(8); // talisman slots
        buffer.u32_from_bool(false); // can equip cloak
        buffer.u32(p_atk.into());
        buffer.u32(p_atk_spd.into());
        buffer.u32(p_def);
        buffer.u32(evasion);
        buffer.u32(accuracy);
        buffer.u32(physical_critical_rate);
        buffer.u32(m_atk.into());
        buffer.u32(cast_spd);
        buffer.u32(p_atk_spd.into());
        buffer.u32(m_def);
        buffer.u32(0); // pvp flag
        buffer.u32(self.pvp_stats.karma);
        buffer.u32(self.base_speed.get(MovementStat::Run));
        buffer.u32(self.base_speed.get(MovementStat::Walk));
        buffer.u32(self.base_speed.get(MovementStat::Swim)); // swim run speed
        buffer.u32(self.base_speed.get(MovementStat::Swim)); // swim walk speed
        buffer.u32(self.base_speed.get(MovementStat::FastFly)); // fly run speed
        buffer.u32(self.base_speed.get(MovementStat::Fly)); // fly walk speed
        buffer.u32(self.base_speed.get(MovementStat::FastFly)); // fly2 run speed
        buffer.u32(self.base_speed.get(MovementStat::Fly)); // fly2 walk speed
        buffer.f64(self.movable.multiplier(&self.base_speed));
        buffer.f64(p_atk_spd_multiplier);
        buffer.f64(self.collision_radius);
        buffer.f64(self.collision_height);
        buffer.u32(self.appearance.hair_style);
        buffer.u32(self.appearance.hair_color);
        buffer.u32(self.appearance.face);
        buffer.u32(1); // GM level
        buffer.str(&self.title);
        buffer.u32(0); // clan id
        buffer.u32(0); // clan crest id
        buffer.u32(0); // ally id
        buffer.u32(0); // ally crest id
        buffer.u32(0); // relation
        buffer.u8(12u8); // mount type
        buffer.u8(0u8); // private store type
        buffer.bool(false); // has dwarven craft
        buffer.u32(self.pvp_stats.pk_kills);
        buffer.u32(self.pvp_stats.pvp_kills);
        buffer.u16(0); // cubics size
        buffer.bool(false); // is in party match room
        buffer.u32_from_bool(false); // is invisible
        buffer.u8_from_usize(self.movable.move_state().into());
        buffer.u32(0); // clan priveleges
        buffer.u16(25); // recommendations left
        buffer.u16(5); // recommendations received
        buffer.u32(5555); // mount npc id
        buffer.u16(100); // inventory limit
        buffer.u32(self.class_id.into());
        buffer.u32(0); // special effects
        buffer.u32(max_cp);
        buffer.u32(current_cp);
        buffer.u8(0u8); // mount effect
        buffer.u8(0u8); // team id
        buffer.u32(0); // clan crest large id
        buffer.bool(false); // noble
        buffer.bool(false); // hero
        buffer.bool(false); // fishing mode
        buffer.i32(0); // fishing x
        buffer.i32(0); // fishing y
        buffer.i32(0); // fishing z
        buffer.u32(u32::MAX); // name color
        buffer.bool(self.movable.is_running()); // running
        buffer.u32(0); // pledge class
        buffer.u32(0); // pledge type
        buffer.u32(300000); // title color
        buffer.u32(0); // cursed weapon level
        buffer.u32(0); // transformation display id
        buffer.u16(1); // attack attribute
        buffer.u16(300); // attack attribute value
        buffer.u16(0); // defence fire
        buffer.u16(0); // defence water
        buffer.u16(0); // defence wind
        buffer.u16(0); // defence earth
        buffer.u16(0); // defence holy
        buffer.u16(0); // defence dark
        buffer.u32(15); // agathion id
        buffer.u32(555); // fame
        buffer.u32(1); // minimap allowed
        buffer.u32(9999); // vitality points
        buffer.u32(0); // abnormal visual effect special
        buffer
    }
}
