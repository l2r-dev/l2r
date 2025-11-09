use super::model::Model;
use crate::{
    character,
    items::{self, DollSlot},
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

#[derive(Clone, Debug, Reflect)]
pub struct CharWithItems {
    pub character: character::Bundle,
    pub items: [items::Id; DollSlot::USER_INFO_COUNT],
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Table {
    characters: Vec<CharWithItems>,
    last_used_slot: Option<CharSlot>,
    selected_slot: Option<CharSlot>,
    character: Option<Entity>,
}

impl Table {
    pub const MAX_CHARACTERS_ON_ACCOUNT: usize = 7;
    pub fn new(characters: Vec<CharWithItems>) -> Self {
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
        world: &mut World,
    ) -> Result<Self, TableError> {
        if char_with_items.len() > Self::MAX_CHARACTERS_ON_ACCOUNT {
            return Err(TableError::MaxCharsReached)?;
        }

        let mut chars_with_items = Vec::with_capacity(Self::MAX_CHARACTERS_ON_ACCOUNT);
        let mut last_used_slot = None;

        char_with_items.sort_by(|a, b| a.0.created_time.cmp(&b.0.created_time));

        for (index, (char, db_items)) in char_with_items.into_iter().enumerate() {
            if char.is_last_active {
                last_used_slot = Some(CharSlot(index as u32));
            }

            let items: [items::Id; DollSlot::USER_INFO_COUNT] =
                DollSlot::user_info_slots().map(|slot| {
                    db_items
                        .iter()
                        .find(|item| {
                            item.location() == items::ItemLocationVariant::PaperDoll
                                && (DollSlot::try_from(item.location_data as u32).ok()
                                    == Some(slot))
                        })
                        .map(|item| item.item_id())
                        .unwrap_or_default()
                });

            let bundle = character::Bundle::new(char, db_items, session_id, world);

            chars_with_items.push(CharWithItems {
                character: bundle,
                items,
            });
        }

        let mut table = Self::new(chars_with_items);

        if !table.is_empty() {
            table.set_last_used_slot(last_used_slot.unwrap_or(CharSlot(0)))?;
        }

        Ok(table)
    }

    pub fn add_bundle(
        &mut self,
        bundle: character::Bundle,
        items: [items::Id; DollSlot::USER_INFO_COUNT],
    ) -> Result<(), TableError> {
        if self.is_max_len() {
            return Err(TableError::MaxCharsReached)?;
        }

        self.characters.push(CharWithItems {
            character: bundle,
            items,
        });
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
            Ok(&self.characters[selected_slot.0 as usize].character)
        } else {
            Err(TableError::InvalidCharSlot)?
        }
    }

    fn get_mut(&mut self) -> Result<&mut CharWithItems, TableError> {
        if let Some(selected_slot) = self.selected_slot {
            Ok(&mut self.characters[selected_slot.0 as usize])
        } else {
            Err(TableError::InvalidCharSlot)?
        }
    }

    pub fn update_bundle(
        &mut self,
        query_item: &character::query::QueryItem,
        items_query: &items::ItemsQuery,
    ) {
        let char_with_items = match self.get_mut() {
            Ok(char_with_items) => char_with_items,
            Err(_) => {
                log::error!("No character selected to update");
                return;
            }
        };

        char_with_items.character.update(query_item);
        char_with_items.items = query_item
            .paperdoll
            .user_info_iter()
            .map(|slot_item| {
                slot_item
                    .object_id
                    .and_then(|object_id| {
                        items_query
                            .item_by_object_id(object_id)
                            .ok()
                            .map(|item| item.id())
                    })
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect(DollSlot::USER_INFO_COUNT_ERR);
    }

    pub fn all(&self) -> impl Iterator<Item = &CharWithItems> {
        self.characters.iter()
    }

    pub fn all_bundles(&self) -> impl Iterator<Item = &character::Bundle> {
        self.characters.iter().map(|c| &c.character)
    }

    pub fn is_valid_slot(&self, char_slot: CharSlot) -> bool {
        (char_slot.0 as usize) < self.len()
    }

    pub fn remove_slot(&mut self, char_slot: CharSlot) -> Result<CharWithItems, TableError> {
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
