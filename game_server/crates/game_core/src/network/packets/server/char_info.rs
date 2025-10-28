use super::GameServerPacketCodes;
use crate::{character, items::PaperDoll, object_id::ObjectId, stats::*};
use bevy::prelude::*;
use core::fmt;
use l2r_core::{
    model::{base_class::BaseClass, race::Race},
    packets::{L2rServerPacket, ServerPacketBuffer},
};
use spatial::{GameVec3, Heading};

#[derive(Event, Reflect)]
pub struct SendCharInfo;

#[derive(Clone, Reflect)]
pub struct CharInfo {
    pub name: String,
    pub title: String,
    pub race: Race,
    pub base_class: BaseClass,
    pub class_id: ClassId,
    pub object_id: ObjectId,
    pub progress_level: ProgressLevelStats,
    pub primal_stats: PrimalStats,
    pub attack_stats: AttackStats,
    pub pvp_stats: PvpStats,
    pub movable: Movable,
    pub base_speed: MovementStats,
    pub appearance: character::Appearance,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub transform: Transform,
    pub paperdoll_items: PaperDoll,
    pub in_combat: bool,
    pub invisible: bool,
    pub dead: bool,
    pub standing: bool,
    pub in_party_match_room: bool,
    //TODO: для дебага
    pub entity: Entity,
}
impl fmt::Debug for CharInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:?}> {:?} - {:?} lvl, current_pos: {:?}",
            self.object_id,
            self.name,
            self.progress_level.level(),
            GameVec3::from(self.transform.translation)
        )
    }
}
impl L2rServerPacket for CharInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let rotation_heading = Heading::from(self.transform.rotation);

        let mut buffer = ServerPacketBuffer::default();

        let converted_pos = GameVec3::from(self.transform.translation);

        let p_atk_spd = self.attack_stats.typed::<PAtkSpd>(AttackStat::PAtkSpd);
        let cast_spd = self.attack_stats.typed::<CastSpd>(AttackStat::CastSpd);

        buffer.extend(GameServerPacketCodes::CHAR_INFO.to_le_bytes());
        buffer.extend(converted_pos.to_le_bytes());
        buffer.u32(0); // vehicle id
        buffer.u32(u32::from(self.object_id));
        buffer.str(&self.name);
        buffer.u32(self.race.into());
        buffer.u32(self.appearance.gender.into());
        buffer.u32(self.class_id.into());

        for slot_item in self.paperdoll_items.char_info_iter() {
            buffer.u32(
                slot_item
                    .unique_item()
                    .map(|u| u.item().id().into())
                    .unwrap_or(0),
            );
        }

        for _slot_item in self.paperdoll_items.char_info_iter() {
            // augumentation id
            buffer.u32(0);
        }
        buffer.u32(7); // max talisman slots
        buffer.u32_from_bool(false); // max cloak slots (or can equip cloak)
        buffer.u32_from_bool(self.pvp_stats.pvp_flag);
        buffer.u32(self.pvp_stats.karma);
        buffer.u32(cast_spd.into());
        buffer.u32(p_atk_spd.into());
        buffer.u32(0); // padding?
        buffer.u32(self.base_speed.get(MovementStat::Run));
        buffer.u32(self.base_speed.get(MovementStat::Walk));
        buffer.u32(self.base_speed.get(MovementStat::FastSwim));
        buffer.u32(self.base_speed.get(MovementStat::Swim));
        buffer.u32(self.base_speed.get(MovementStat::FastFly)); // fly run speed
        buffer.u32(self.base_speed.get(MovementStat::Fly)); // fly walk speed
        buffer.u32(self.base_speed.get(MovementStat::FastFly)); // fly2? run speed
        buffer.u32(self.base_speed.get(MovementStat::Fly)); // fly2? walk speed
        buffer.f64(self.movable.multiplier(&self.base_speed));
        buffer.f64(p_atk_spd.get_attack_speed_multiplier());
        buffer.f64(self.collision_radius);
        buffer.f64(self.collision_height);
        buffer.u32(self.appearance.hair_style);
        buffer.u32(self.appearance.hair_color);
        buffer.u32(self.appearance.face);
        //TODO: для дебага
        buffer.str(&format!(
            "{} {} {}",
            self.title, self.object_id, self.entity
        ));
        buffer.u32(0); // clan id
        buffer.u32(0); // clan crest id
        buffer.u32(0); // ally id
        buffer.u32(0); // ally crest id
        buffer.bool(self.standing); // standing = 1 sitting = 0
        buffer.bool(self.movable.is_running()); // running = 1 walking = 0
        buffer.bool(self.in_combat); // in combat
        buffer.bool(self.dead);
        buffer.bool(self.invisible);
        buffer.u8(0); // mount type 1 - strider, 2 - wyvern, 3 - great wolf, 0 - none
        buffer.u8(0); // private store type
        buffer.u16(0); // cubics size
        // TODO: extend with cubic-ids u16 later
        buffer.bool(self.in_party_match_room);
        buffer.u32(0); // annormal visual effect
        buffer.u8_from_usize(self.movable.move_state().into());
        buffer.u16(0); // reccomendations left
        buffer.u32(1000000); // mount npcid
        buffer.u32(self.class_id.into());
        buffer.u32(0); // 0 padding
        buffer.u8(0); // enchant effect
        buffer.u8(0); // team id
        buffer.u32(0); // clan crest large id
        buffer.bool(false); // noble
        buffer.bool(false); // hero
        buffer.bool(false); // fishing mode
        buffer.i32(0); // fishing x
        buffer.i32(0); // fishing y
        buffer.i32(0); // fishing z
        buffer.u32(u32::MAX); // name color
        buffer.i32(rotation_heading.into());
        buffer.u32(0); // pledge class
        buffer.u32(0); // pledge type
        buffer.u32(0); // title color
        buffer.u32(0); // cursed weapon level
        buffer.u32(0); // reputation score
        buffer.u32(0); // transformation display id
        buffer.u32(0); // agathion id
        buffer.u32(0); // pad 0
        buffer.u32(0); // abnormal visual effect special
        buffer
    }
}

impl CharInfo {
    pub fn new(query: &character::QueryItem, base_speed: MovementStats) -> Self {
        let invisible = !matches!(query.visibility, Visible::Visible);

        let collision_radius = query.collider.radius();
        let collision_height = query.collider.height();

        CharInfo {
            name: query.name.to_string(),
            title: query.title.to_string(),
            race: *query.race,
            base_class: *query.base_class,
            class_id: query.sub_class.class_id(),
            object_id: *query.object_id,
            progress_level: query.progress_level.clone(),
            primal_stats: query.primal_stats.clone(),
            attack_stats: query.attack_stats.clone(),
            pvp_stats: *query.pvp_stats,
            movable: query.movable.clone(),
            base_speed,
            appearance: *query.appearance,
            collision_radius,
            collision_height,
            transform: *query.transform,
            paperdoll_items: query.paperdoll.clone(),
            dead: query.dead.is_some(),
            in_combat: query.in_combat.is_some(),
            in_party_match_room: false,
            invisible,
            standing: query.sitting.is_none(),
            entity: query.entity,
        }
    }
}
