use bevy::prelude::*;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum::Display;

pub const DROP_DEFAULT_CAPACITY: usize = 4;

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
pub struct DropTable(pub Vec<DropList>);
impl DropTable {
    pub fn calculate_drops(&self) -> SmallVec<[(super::Id, u64); DROP_DEFAULT_CAPACITY]> {
        let mut items_to_spawn = SmallVec::new();
        let mut rng = thread_rng();

        for drop_list in &self.0 {
            if let Some(group) = &drop_list.group {
                let random_value: f32 = rng.gen_range(0.0..=100.0);

                if random_value <= group.chance {
                    for drop_item in &group.items {
                        if rng.gen_range(0.0..=100.0) <= drop_item.chance {
                            let count = if drop_item.min == drop_item.max {
                                drop_item.min
                            } else {
                                rng.gen_range(drop_item.min..=drop_item.max)
                            };

                            items_to_spawn.push((drop_item.id, count));
                        }
                    }
                }
            }
        }

        items_to_spawn
    }

    pub fn calculate_spoils(&self) -> SmallVec<[(super::Id, u64); DROP_DEFAULT_CAPACITY]> {
        let mut items_to_spawn = SmallVec::with_capacity(DROP_DEFAULT_CAPACITY);
        let mut rng = thread_rng();

        for drop_list in &self.0 {
            if let Some(spoil) = &drop_list.spoil {
                for drop_item in &spoil.0 {
                    if rng.gen_range(0.0..=100.0) <= drop_item.chance {
                        let count = if drop_item.min == drop_item.max {
                            drop_item.min
                        } else {
                            rng.gen_range(drop_item.min..=drop_item.max)
                        };

                        items_to_spawn.push((drop_item.id, count));
                    }
                }
            }
        }

        items_to_spawn
    }

    pub fn get_drop_items_json(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::with_capacity(self.0.len());
        for drop_list in &self.0 {
            if let Some(group) = &drop_list.group {
                for item in &group.items {
                    result.push(serde_json::json!({
                        "id": item.id,
                        "min": item.min,
                        "max": item.max,
                        "chance": item.chance,
                        "description": item.description
                    }));
                }
            }
        }
        result
    }

    pub fn get_spoil_items_json(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::with_capacity(self.0.len());
        for drop_list in &self.0 {
            if let Some(spoil) = &drop_list.spoil {
                for item in &spoil.0 {
                    result.push(serde_json::json!({
                        "id": item.id,
                        "min": item.min,
                        "max": item.max,
                        "chance": item.chance,
                        "description": item.description
                    }));
                }
            }
        }
        result
    }
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ItemMetric {
    ItemsDropped,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DropList {
    pub group: Option<DropGroup>,
    pub spoil: Option<Spoil>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DropGroup {
    pub chance: f32,
    pub items: Vec<DropItem>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Spoil(pub Vec<DropItem>);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DropItem {
    pub id: super::Id,
    pub min: u64,
    pub max: u64,
    pub chance: f32,
    pub description: String,
}
