use super::GameServerPacketCodes;
use crate::{
    character::{self, DeleteTimer},
    items,
    network::packets::client::CharSlot,
    object_id::ObjectId,
    stats::*,
};
use bevy::{platform::collections::HashMap, prelude::*};
use core::fmt;
use l2r_core::{
    model::{race::Race, session::SessionId},
    packets::{L2rServerPacket, ServerPacketBuffer},
};
use spatial::GameVec3;

#[derive(Clone, Default, Deref, Reflect)]
pub struct CharSelectionInfo(Vec<CharInfoData>);

impl fmt::Debug for CharSelectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl L2rServerPacket for CharSelectionInfo {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.extend(GameServerPacketCodes::CHARACTER_SELECTION_INFO.to_le_bytes());
        buffer.u32_from_usize(self.len());
        buffer.u32(character::Table::MAX_CHARACTERS_ON_ACCOUNT as u32);
        buffer.u8(0);
        for char in self.iter() {
            buffer.extend(char.buffer().to_vec());
        }
        buffer
    }
}

impl CharSelectionInfo {
    pub fn new(
        chars_table: &character::Table,
        models: Vec<(
            character::model::Model,
            HashMap<ObjectId, items::model::Model>,
        )>,
    ) -> Self {
        let mut chars = Vec::with_capacity(chars_table.len());
        for (index, char) in chars_table.all().iter().enumerate() {
            let last_used = chars_table.last_used_slot() == Some(CharSlot(index as u32));

            // Find the matching character model by ObjectId
            let char_model = models.iter().find(|(model, _)| model.id == char.id);

            let equipped_items: Vec<items::Id> = char
                .paper_doll
                .user_info_iter()
                .map(|slot_item| {
                    slot_item
                        .object_id
                        .and_then(|id| {
                            char_model
                                .and_then(|(_, items_map)| items_map.get(&id))
                                .map(|item| item.item_id())
                        })
                        .unwrap_or_default()
                })
                .collect();

            chars.push(CharInfoData::new(char, equipped_items, last_used));
        }

        Self(chars)
    }

    pub fn from_query(chars_table: &character::Table, items_query: &items::ItemsQuery) -> Self {
        let mut chars = Vec::with_capacity(chars_table.len());
        for (index, char) in chars_table.all().iter().enumerate() {
            let last_used = chars_table.last_used_slot() == Some(CharSlot(index as u32));

            let equipped_items: Vec<items::Id> = char
                .paper_doll
                .user_info_iter()
                .map(|slot_item| {
                    slot_item
                        .object_id
                        .and_then(|id| items_query.item_by_object_id(id).ok().map(|item| item.id()))
                        .unwrap_or_default()
                })
                .collect();

            chars.push(CharInfoData::new(char, equipped_items, last_used));
        }
        Self(chars)
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct CharInfoData {
    pub name: String,
    pub title: String,
    pub session_id: SessionId,
    pub clan_id: u32,
    pub builder_level: u32,
    pub race: Race,
    pub class_id: ClassId,
    pub is_active: bool,
    pub position: Vec3,
    pub vitals: VitalsStats,
    pub progress_stats: ProgressStats,
    pub progress_level: ProgressLevelStats,
    pub pvp: PvpStats,
    pub paperdoll_item_ids: Vec<items::Id>,
    pub appearance: character::Appearance,
    pub delete_timer: DeleteTimer,
    pub is_last_used: bool,
    pub enchant_effect: u8,
    pub augmentation_id: u32,
    pub transform_id: u32,
    pub pet_npc_id: u32,
    pub pet_level: u32,
    pub pet_food: u32,
    pub pet_food_level: u32,
    pub pet_hp: f64,
    pub pet_mp: f64,
}

impl CharInfoData {
    pub fn buffer(&self) -> ServerPacketBuffer {
        let max_hp = self.vitals.get(VitalsStat::MaxHp) as f64;
        let current_hp = self.vitals.get(VitalsStat::Hp) as f64;
        let max_mp = self.vitals.get(VitalsStat::MaxMp) as f64;
        let current_mp = self.vitals.get(VitalsStat::Mp) as f64;
        let char_level = self.progress_level.level();

        let mut buffer = ServerPacketBuffer::default();
        buffer.str(&self.name);
        buffer.u32(0);
        buffer.str(&self.title);
        buffer.u32_from_usize(*self.session_id);
        buffer.u32(self.clan_id);
        buffer.u32(self.builder_level);
        buffer.u32(self.appearance.gender.into());
        buffer.u32(self.race.into());
        buffer.u32(self.class_id.into());
        buffer.u32_from_bool(self.is_active);
        buffer.extend(GameVec3::from(self.position).to_le_bytes());
        buffer.f64(current_hp);
        buffer.f64(current_mp);
        buffer.u32(self.progress_stats.sp());
        buffer.u64(self.progress_stats.exp());
        buffer.f64(self.progress_stats.exp_percent(char_level));
        buffer.u32(char_level.into());
        buffer.u32(self.pvp.karma);
        buffer.u32(self.pvp.pk_kills);
        buffer.u32(self.pvp.pvp_kills);
        // unknown 7 blocks of 4 bytes
        for _ in 0..7 {
            buffer.u32(0);
        }
        for item_id in self.paperdoll_item_ids.iter().copied() {
            buffer.u32(item_id.into());
        }
        buffer.u32(self.appearance.hair_style);
        buffer.u32(self.appearance.hair_color);
        buffer.u32(self.appearance.face);
        buffer.f64(max_hp);
        buffer.f64(max_mp);
        buffer.u32(self.delete_timer.into());
        buffer.u32(self.class_id.into());
        buffer.u32_from_bool(self.is_last_used);
        buffer.u8(self.enchant_effect);
        buffer.u32(self.augmentation_id);
        buffer.u32(self.transform_id);
        buffer.u32(self.pet_npc_id);
        buffer.u32(self.pet_level);
        buffer.u32(self.pet_food);
        buffer.u32(self.pet_food_level);
        buffer.f64(self.pet_hp);
        buffer.f64(self.pet_mp);
        buffer.u32(self.progress_stats.get(ProgressStat::VitalityPoints) as u32);
        buffer
    }
}

impl CharInfoData {
    fn new(
        bundle: &character::Bundle,
        paperdoll_item_ids: Vec<items::Id>,
        is_last_used: bool,
    ) -> Self {
        Self {
            name: bundle.name.to_string(),
            title: bundle.title.to_string(),
            session_id: bundle.session_id,
            clan_id: 0,
            builder_level: 0,
            appearance: bundle.appearance,
            race: bundle.race,
            class_id: bundle.sub_class.class_id(),
            is_active: true,
            position: bundle.transform.translation,
            vitals: bundle.vitals_stats.clone(),
            progress_stats: bundle.progress_stats.clone(),
            progress_level: bundle.progress_level.clone(),
            pvp: bundle.pvp,
            paperdoll_item_ids,
            delete_timer: bundle.delete_timer,
            is_last_used,
            enchant_effect: 0,
            augmentation_id: 0,
            transform_id: 0,
            pet_npc_id: 0,
            pet_level: 0,
            pet_food: 0,
            pet_food_level: 0,
            pet_hp: 0.0,
            pet_mp: 0.0,
        }
    }
}

#[derive(Event)]
pub struct SendCharSelectionInfo;
