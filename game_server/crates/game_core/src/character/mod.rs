use bevy::{
    app::{App, Plugin},
    ecs::component::Component,
    platform::collections::HashMap,
    prelude::Entity,
    reflect::Reflect,
};
use l2r_core::plugins::custom_hierarchy::HierarchyFolderOperations;
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
    hierarchy_folders: HashMap<TypeId, Entity>,
}

impl HierarchyFolderOperations for Character {
    fn get_folder<T: Component>(&self) -> Option<Entity> {
        self.hierarchy_folders.get(&TypeId::of::<T>()).copied()
    }

    fn set_folder<T: Component>(&mut self, folder_entity: Entity) {
        self.hierarchy_folders
            .insert(TypeId::of::<T>(), folder_entity);
    }

    fn remove_folder<T: Component>(&mut self) {
        self.hierarchy_folders.remove(&TypeId::of::<T>());
    }

    fn folders_iter(&self) -> impl Iterator<Item = (TypeId, bevy::prelude::Entity)> {
        self.hierarchy_folders.iter().map(|(k, v)| (*k, *v))
    }
}

#[derive(Clone, Component, Copy, Debug, Default, Reflect)]
pub struct CharacterItemsFolder;
