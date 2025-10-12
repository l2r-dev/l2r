use crate::stats::ClassId;
use bevy::{platform::collections::HashMap, prelude::*};
use l2r_core::model::base_class::BaseClass;
use serde::Deserialize;

pub const CLASS_TREE_ASSETS_ERROR: &str =
    "Failed to load ClassTree asset, must be loaded on startup";

#[derive(Default, Deref, DerefMut)]
pub struct ClassTreeHandle(Handle<ClassTree>);

#[derive(Asset, Clone, Debug, Default, Deserialize, Resource, TypePath)]
pub struct ClassTree(HashMap<ClassId, Option<ClassId>>);

impl ClassTree {
    fn get_base_class_id(&self, id: ClassId) -> ClassId {
        let mut current_id = id;
        while let Some(class_stats) = self.0.get(&current_id) {
            if let Some(parent_id) = class_stats {
                current_id = *parent_id;
            } else {
                break;
            }
        }
        current_id
    }

    pub fn get_base_class(&self, class_id: ClassId) -> BaseClass {
        let base_class = self.get_base_class_id(class_id);
        match base_class {
            ClassId::HumanFighter => BaseClass::Fighter,
            ClassId::HumanMystic => BaseClass::Mystic,
            ClassId::ElvenFighter => BaseClass::Fighter,
            ClassId::ElvenMystic => BaseClass::Mystic,
            ClassId::DarkFighter => BaseClass::Fighter,
            ClassId::DarkMystic => BaseClass::Mystic,
            ClassId::OrcFighter => BaseClass::Fighter,
            ClassId::OrcMystic => BaseClass::Mystic,
            ClassId::DwarvenFighter => BaseClass::Fighter,
            ClassId::SoldierMale => BaseClass::MaleFighter,
            ClassId::SoldierFemale => BaseClass::FemaleFighter,
            _ => BaseClass::default(),
        }
    }
}
