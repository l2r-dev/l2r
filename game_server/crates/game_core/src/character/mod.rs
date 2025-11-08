use bevy::{
    app::{App, Plugin},
    ecs::component::Component,
    platform::collections::HashMap,
    prelude::Entity,
    reflect::Reflect,
};
use std::any::TypeId;

pub mod model;
pub mod skills;

mod appearance;
mod bundle;
mod delete_timer;
mod query;
mod table;

pub use appearance::*;
pub use bundle::*;
pub use delete_timer::*;
pub use model::CharacterRepository;
pub use query::*;
pub use table::*;

#[derive(Debug, bevy::prelude::Event)]
pub struct CharacterSave;

pub struct CharacterComponentsPlugin;
impl Plugin for CharacterComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<model::Model>()
            .register_type::<skills::Model>()
            .register_type::<Table>();

        app.add_event::<CharacterSave>();
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Character {
    folders: HashMap<TypeId, Entity>,
}

impl Character {
    pub fn set_folder<T: Component>(&mut self, entity: Entity) {
        self.folders.insert(TypeId::of::<T>(), entity);
    }

    pub fn folders(&self) -> &HashMap<TypeId, Entity> {
        &self.folders
    }
}

#[derive(Clone, Component, Copy, Debug, Default, Reflect)]
pub struct ItemsFolder;
