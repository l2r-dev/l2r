use super::GameServerPacketCodes;
use crate::{
    items::{self, DollSlot, ItemsQuery, PaperDoll},
    npc,
    object_id::ObjectId,
    stats::{Stats, *},
};
use avian3d::parry::shape::Capsule;
use bevy::prelude::*;
use core::fmt;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use spatial::{GameVec3, Heading};

#[derive(Clone, Reflect)]
pub struct NpcInfo {
    pub object_id: ObjectId,
    pub npc_id: npc::Id,
    pub display_id: Option<npc::DisplayId>,
    pub is_attackable: bool,
    pub position: GameVec3,
    pub heading: Heading,
    pub m_atk_spd: u32,
    pub p_atk_spd: u32,
    pub run_spd: u32,
    pub walk_spd: u32,
    pub swim_run_spd: u32,
    pub swim_walk_spd: u32,
    pub fly_run_spd: u32,
    pub fly_walk_spd: u32,
    pub move_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub rhand_item: items::Id,
    pub chest_item: items::Id,
    pub lhand_item: items::Id,
    pub name_above: bool,
    pub is_running: bool,
    pub in_combat: bool,
    pub is_alike_dead: bool,
    pub is_summoned: bool,
    pub name: String,
    pub title: String,
    pub title_color: u32,
    pub pvp_flag: bool,
    pub karma: u32,
    pub abnormal_visual_effect: u32,
    pub clan_id: u32,
    pub crest_id: u32,
    pub ally_id: u32,
    pub ally_crest_id: u32,
    pub move_state: MoveState,
    pub team_id: u8,
    pub enchant_effect: u32,
    pub is_flying: u32,
    pub color_effect: u32,
    pub abnormal_visual_effect_special: u32,
    //TODO: для дебага
    pub entity: Entity,
}

impl fmt::Debug for NpcInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}], npc_id={:?}, name={:?}, position={:?}, dead={:?}",
            self.object_id, self.npc_id, self.name, self.position, self.is_alike_dead
        )
    }
}

impl L2rServerPacket for NpcInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();

        let npc_id = self.display_id.as_deref().unwrap_or(&self.npc_id);

        buffer.extend(GameServerPacketCodes::NPC_INFO.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(u32::from(*npc_id) + 1000000);
        buffer.u32_from_bool(self.is_attackable);
        buffer.extend(self.position.to_le_bytes());
        buffer.i32(self.heading.into());
        buffer.u32(0);
        buffer.u32(self.m_atk_spd);
        buffer.u32(self.p_atk_spd);
        buffer.u32(self.run_spd);
        buffer.u32(self.walk_spd);
        buffer.u32(self.swim_run_spd);
        buffer.u32(self.swim_walk_spd);
        buffer.u32(self.fly_run_spd);
        buffer.u32(self.fly_walk_spd);
        buffer.u32(self.fly_run_spd);
        buffer.u32(self.fly_walk_spd);
        buffer.f64(self.move_multiplier);
        buffer.f64(self.attack_speed_multiplier);
        buffer.f64(self.collision_radius);
        buffer.f64(self.collision_height);
        buffer.u32(self.rhand_item.into());
        buffer.u32(self.chest_item.into());
        buffer.u32(self.lhand_item.into());
        buffer.bool(self.name_above);
        buffer.bool(self.is_running);
        buffer.bool(self.in_combat);
        buffer.bool(self.is_alike_dead);
        buffer.bool(self.is_summoned);
        buffer.i32(-1);
        buffer.str(&self.name);
        buffer.i32(-1);
        //TODO: для дебага
        buffer.str(&format!(
            "{} {} {}",
            self.title, self.object_id, self.entity
        ));
        buffer.u32(self.title_color);
        buffer.u32_from_bool(self.pvp_flag);
        buffer.u32(self.karma);
        buffer.u32(self.abnormal_visual_effect);
        buffer.u32(self.clan_id);
        buffer.u32(self.crest_id);
        buffer.u32(self.ally_id);
        buffer.u32(self.ally_crest_id);
        buffer.u8_from_usize(self.move_state.into());
        buffer.u8(self.team_id);
        buffer.f64(self.collision_radius);
        buffer.f64(self.collision_height);
        buffer.u32(self.enchant_effect);
        buffer.u32(0); // is_flying?
        buffer.u32(0); // 0
        buffer.u32(self.color_effect);
        buffer.u8(1);
        buffer.u8(1);
        buffer.u32(self.abnormal_visual_effect_special);
        buffer.u32(0);
        buffer
    }
}

impl NpcInfo {
    pub fn new(npc: &npc::NpcQueryItem, items: &ItemsQuery, base_speed: MovementStats) -> Self {
        let heading = Heading::from(npc.transform.rotation);
        let default_doll = PaperDoll::default();
        let paperdoll_items = npc.paperdoll_items.unwrap_or(&default_doll);

        let collision_radius = npc
            .collider
            .shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.radius)
            .unwrap_or(0.0) as f64;
        let collision_height = npc
            .collider
            .shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.segment.length())
            .unwrap_or(0.0) as f64;

        let m_atk_spd = npc.attack_stats.typed::<CastSpd>(AttackStat::CastSpd);
        let p_atk_spd = npc.attack_stats.typed::<PAtkSpd>(AttackStat::PAtkSpd);
        let attack_speed_multiplier = p_atk_spd.get_attack_speed_multiplier();

        let rhand_oid = paperdoll_items.get(DollSlot::RightHand);
        let rhand_item = rhand_oid
            .and_then(|oid| Some(items.item_by_object_id(oid).ok()?.id()))
            .unwrap_or_default();
        let lhand_oid = paperdoll_items.get(DollSlot::LeftHand);
        let lhand_item = lhand_oid
            .and_then(|oid| Some(items.item_by_object_id(oid).ok()?.id()))
            .unwrap_or_default();
        let chest_oid = paperdoll_items.get(DollSlot::Chest);
        let chest_item = chest_oid
            .and_then(|oid| Some(items.item_by_object_id(oid).ok()?.id()))
            .unwrap_or_default();

        NpcInfo {
            object_id: *npc.object_id,
            npc_id: *npc.id,
            display_id: npc.display_id.copied(),
            is_attackable: npc.attackable.is_some(),
            position: GameVec3::from(npc.transform.translation),
            heading,
            m_atk_spd: *m_atk_spd,
            p_atk_spd: *p_atk_spd,
            run_spd: base_speed.get(MovementStat::Run),
            walk_spd: base_speed.get(MovementStat::Walk),
            swim_run_spd: base_speed.get(MovementStat::FastSwim),
            swim_walk_spd: base_speed.get(MovementStat::Swim),
            fly_run_spd: base_speed.get(MovementStat::FastFly),
            fly_walk_spd: base_speed.get(MovementStat::Fly),
            move_multiplier: npc.movable.multiplier(&base_speed),
            attack_speed_multiplier,
            collision_radius,
            collision_height,
            rhand_item,
            lhand_item,
            chest_item,
            name_above: true,
            is_running: npc.movable.running(),
            in_combat: npc.in_combat.is_some(),
            is_alike_dead: npc.dead.is_some(),
            is_summoned: npc.summon.is_some(),
            is_flying: 0,
            name: npc.name.to_string(),
            title: npc.title.map_or(String::new(), |t| t.to_string()),
            title_color: 0,
            pvp_flag: npc.pvp_stats.pvp_flag,
            karma: npc.pvp_stats.karma,
            abnormal_visual_effect: 0,
            clan_id: 0,
            crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            move_state: npc.movable.move_state(),
            team_id: 0,
            enchant_effect: 0,
            color_effect: 0,
            abnormal_visual_effect_special: 0,
            entity: npc.entity,
        }
    }
}
