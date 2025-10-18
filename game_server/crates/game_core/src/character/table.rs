use super::model::Model;
use crate::{
    character,
    items::{self, PaperDoll, UniqueItem},
    network::packets::client::CharSlot,
};
use bevy::{log, prelude::*};
use bevy_slinet::connection::ConnectionId;
use l2r_core::model::session::SessionId;
use std::fmt;

#[derive(Clone, Debug)]
pub enum TableError {
    MaxCharsReached,
    InvalidCharSlot,
}

impl From<TableError> for BevyError {
    fn from(err: TableError) -> Self {
        BevyError::from(format!("Character Table Error: {}", err))
    }
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::MaxCharsReached => write!(f, "Maximum number of characters reached"),
            TableError::InvalidCharSlot => write!(f, "Invalid character slot"),
        }
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Table {
    characters: Vec<character::Bundle>,
    last_used_slot: Option<CharSlot>,
    selected_slot: Option<CharSlot>,
    character: Option<Entity>,
}

impl Table {
    pub const MAX_CHARACTERS_ON_ACCOUNT: usize = 7;
    pub fn new(characters: Vec<character::Bundle>) -> Self {
        Self {
            characters,
            last_used_slot: None,
            selected_slot: None,
            character: None,
        }
    }

    pub fn from_char_list(
        mut char_with_items: Vec<(Model, Vec<items::model::Model>)>,
        session_id: SessionId,
        world: &World,
    ) -> Result<Self, TableError> {
        if char_with_items.len() >= Self::MAX_CHARACTERS_ON_ACCOUNT {
            return Err(TableError::MaxCharsReached)?;
        }

        let mut bundles = Vec::with_capacity(Self::MAX_CHARACTERS_ON_ACCOUNT);
        let mut last_used_slot = None;

        char_with_items.sort_by(|a, b| a.0.created_time.cmp(&b.0.created_time));

        for (index, (char, db_items)) in char_with_items.into_iter().enumerate() {
            if char.is_last_active {
                last_used_slot = Some(CharSlot(index as u32));
            }

            let items_data_table = world.resource::<items::ItemsDataTable>();
            let items_data_assets = world.resource::<Assets<items::ItemsInfo>>();

            // Process items and create paperdoll
            let items = db_items
                .iter()
                .filter_map(|item| {
                    let item_info =
                        items_data_table.get_item_info(item.item_id(), items_data_assets);
                    if let Ok(item_info) = item_info {
                        Some(UniqueItem::from_model(*item, item_info))
                    } else {
                        log::warn!(
                            "CharTable: No item info found for item id {:?}",
                            item.item_id()
                        );
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut paperdoll = PaperDoll::default();

            for unique_item in items {
                let Some(bodypart) = unique_item.item().bodypart() else {
                    log::warn!(
                        "CharTable: No bodypart found for item id {:?}",
                        unique_item.item().id()
                    );
                    continue;
                };

                paperdoll.equip(
                    bodypart,
                    Some(unique_item),
                    &items_data_table
                        .get_item_info(unique_item.item().id(), items_data_assets)
                        .expect("should exist"),
                    (&items_data_assets, &items_data_table),
                );
            }

            let bundle = character::Bundle::new(char, paperdoll, session_id, world);

            bundles.push(bundle);
        }

        let mut table = Self::new(bundles);

        if !table.is_empty() {
            table.set_last_used_slot(last_used_slot.unwrap_or(CharSlot(0)))?;
        }

        Ok(table)
    }

    pub fn add_bundle(&mut self, bundle: character::Bundle) -> Result<(), TableError> {
        if self.is_max_len() {
            return Err(TableError::MaxCharsReached)?;
        }

        self.characters.push(bundle);
        self.last_used_slot = Some(CharSlot((self.len() as u32) - 1));
        Ok(())
    }

    pub fn select(&mut self, char_slot: CharSlot) -> Result<(), TableError> {
        if char_slot.0 as usize >= self.len() {
            return Err(TableError::InvalidCharSlot)?;
        }
        self.selected_slot = Some(char_slot);
        self.last_used_slot = Some(char_slot);
        Ok(())
    }

    pub fn selected_slot(&self) -> Option<CharSlot> {
        self.selected_slot
    }

    pub fn last_used_slot(&self) -> Option<CharSlot> {
        self.last_used_slot
    }

    pub fn set_last_used_slot(&mut self, slot: CharSlot) -> Result<(), TableError> {
        if slot.0 as usize >= self.len() {
            return Err(TableError::InvalidCharSlot)?;
        }
        self.last_used_slot = Some(slot);
        Ok(())
    }

    pub fn get_bundle(&self) -> Result<&character::Bundle, TableError> {
        if let Some(selected_slot) = self.selected_slot {
            Ok(&self.characters[selected_slot.0 as usize])
        } else {
            Err(TableError::InvalidCharSlot)?
        }
    }

    fn get_mut(&mut self) -> Result<&mut character::Bundle, TableError> {
        if let Some(selected_slot) = self.selected_slot {
            Ok(&mut self.characters[selected_slot.0 as usize])
        } else {
            Err(TableError::InvalidCharSlot)?
        }
    }

    pub fn update_bundle(&mut self, query_item: &character::query::QueryItem) {
        let bundle = match self.get_mut() {
            Ok(bundle) => bundle,
            Err(_) => {
                log::error!("No character selected to update");
                return;
            }
        };
        bundle.update(query_item);
    }

    pub fn all(&self) -> &Vec<character::Bundle> {
        &self.characters
    }

    pub fn is_valid_slot(&self, char_slot: CharSlot) -> bool {
        (char_slot.0 as usize) < self.len()
    }

    pub fn remove_slot(&mut self, char_slot: CharSlot) -> Result<character::Bundle, TableError> {
        if char_slot.0 as usize >= self.len() {
            return Err(TableError::InvalidCharSlot)?;
        }
        Ok(self.characters.remove(char_slot.0 as usize))
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    pub fn is_max_len(&self) -> bool {
        self.len() >= Self::MAX_CHARACTERS_ON_ACCOUNT
    }

    pub fn set_character(&mut self, entity: Entity) {
        self.character = Some(entity);
    }

    pub fn unset_character(&mut self) {
        self.character = None;
    }

    pub fn character(&self) -> Result<Entity> {
        self.character
            .ok_or_else(|| BevyError::from("Character not found"))
    }
}

pub trait FromCharListBuilder {
    fn from_char_list(
        char_with_items: Vec<(Model, Vec<items::model::Model>)>,
        session_id: ConnectionId,
        world: &World,
    ) -> Result<Table, TableError>;
}
