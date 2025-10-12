use crate::items;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::{Deserialize, Serialize};
use spatial::GameVec3;

pub struct TeleportDestinationsComponentsPlugin;
impl Plugin for TeleportDestinationsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<TeleportDestinations>::new(&["json"]));

        app.register_type::<TeleportDestinationsHandle>()
            .register_type::<TeleportDestination>();
    }
}

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
pub struct TeleportDestinationsHandle(Handle<TeleportDestinations>);

#[derive(Asset, Clone, Debug, Deref, Deserialize, Eq, PartialEq, Reflect, Serialize)]
pub struct TeleportDestinations(HashMap<super::Id, TeleportDestination>);
impl TeleportDestinations {
    pub fn get_many(
        &self,
        ids: impl IntoIterator<Item = super::Id>,
    ) -> Vec<(super::Id, &TeleportDestination)> {
        ids.into_iter()
            .filter_map(|id| self.0.get(&id).map(|dest| (id, dest)))
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = (super::Id, &TeleportDestination)> {
        self.0.iter().map(|(id, dest)| (*id, dest))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Reflect, Serialize)]
pub struct TeleportDestination {
    pub name: String,
    pub location: GameVec3,
    #[serde(default)]
    pub price: u64,
    #[serde(default)]
    pub item: items::Id,
    #[serde(default)]
    pub noble: bool,
}

#[derive(Serialize)]
pub struct TeleportDestinationTemplate<'a> {
    pub(super) id: super::Id,
    pub(super) dest: TeleportDestination,
    pub(super) item_name: &'a str,
}

impl<'a> TeleportDestinationTemplate<'a> {
    pub fn new(
        id: super::Id,
        destination: TeleportDestination,
        item_name: &'a str,
    ) -> TeleportDestinationTemplate<'a> {
        TeleportDestinationTemplate {
            id,
            dest: destination,
            item_name,
        }
    }
}
