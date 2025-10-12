use crate::items::Item;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect)]
pub struct Good {
    pub item: Item,
    pub tax: bool,
    pub keep: bool,
}

impl Good {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            tax: false,
            keep: false,
        }
    }
}
