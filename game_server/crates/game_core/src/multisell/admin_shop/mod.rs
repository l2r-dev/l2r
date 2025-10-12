use crate::items::{Grade, Kind};
use bevy::{platform::collections::HashMap, prelude::*};
use serde::Serialize;

mod id_gen;

pub struct AdminShopComponentsPlugin;
impl Plugin for AdminShopComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AdminShopUpdate>()
            .register_type::<AdminShopMultiSells>();

        app.add_event::<AdminShopUpdate>();
        app.init_resource::<AdminShopMultiSells>();
    }
}

#[derive(Clone, Copy, Debug, Event, Reflect)]
pub struct AdminShopUpdate;

#[derive(Clone, Debug, Serialize)]
pub struct NamedMultisell {
    id: super::Id,
    name: String,
    grade: Grade,
}

impl NamedMultisell {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn grade(&self) -> &Grade {
        &self.grade
    }

    pub fn id(&self) -> super::Id {
        self.id
    }
}

impl From<(Kind, Grade)> for NamedMultisell {
    fn from((kind, grade): (Kind, Grade)) -> Self {
        let name = if grade == Grade::None {
            kind.to_string()
        } else {
            format!("{} grade", grade)
        };
        Self {
            id: (kind, grade).into(),
            name,
            grade,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CategoryMultisell {
    name: String,
    subcategories: Vec<NamedMultisell>,
}

impl CategoryMultisell {
    pub fn new(name: String, subcategories: Vec<NamedMultisell>) -> Self {
        Self {
            name,
            subcategories,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn subcategories(&self) -> &[NamedMultisell] {
        &self.subcategories
    }
}

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
#[reflect(Resource)]
pub struct AdminShopMultiSells(#[reflect(ignore)] HashMap<super::Id, Vec<super::Entry>>);
