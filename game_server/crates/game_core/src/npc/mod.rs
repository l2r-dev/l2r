use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_ecs::system::SystemParam;
use derive_more::From;
use l2r_core::{assets::ASSET_DIR, utils::get_base_path};
use map::{WorldMap, id::RegionId};
use serde::Deserialize;
use serde_json::from_reader;
use std::{fs::File, io::BufReader};

mod bundle;
mod commands;
mod dialog;
mod id;
pub mod kind;
mod model;
mod monster_ai;
mod query;
mod summon;

pub use bundle::Bundle;
pub use commands::*;
pub use dialog::*;
pub use id::*;
pub use kind::Kind;
pub use model::Model;
pub use monster_ai::*;
pub use query::{NpcQuery, NpcQueryItem};

pub struct NpcComponentsPlugin;
impl Plugin for NpcComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<NpcInfo>::new(&["json"]));

        app.add_event::<Spawn>()
            .add_event::<Spawned>()
            .add_event::<GenerateDropRequest>();

        app.register_type::<Id>()
            .register_type::<Kind>()
            .register_type::<RegionalNpcHandles>();
    }
}

#[derive(Clone, Copy, Debug, Event, From)]
pub struct GenerateDropRequest;

#[derive(Asset, Clone, Debug, Default, Deref, DerefMut, Deserialize, Resource, TypePath)]
pub struct NpcInfo(HashMap<Id, Model>);
impl NpcInfo {
    pub fn test_data() -> Self {
        let mut asset_dir = get_base_path();
        asset_dir.push(ASSET_DIR);

        let path = asset_dir.join("tests\\npc.json");

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {path:?}"));

        let reader = BufReader::new(file);

        from_reader(reader).unwrap_or_else(|_| panic!("Failed to parse from JSON: {path:?}"))
    }
}

#[derive(SystemParam)]
pub struct RegionalNpcInfoQuery<'w, 's> {
    pub world_map: Res<'w, WorldMap>,
    pub regions: Query<'w, 's, Ref<'static, RegionalNpcHandles>>,
    pub transforms: Query<'w, 's, (Ref<'static, Id>, Ref<'static, Transform>)>,
    pub assets: Res<'w, Assets<NpcInfo>>,
}

impl<'w, 's> RegionalNpcInfoQuery<'w, 's> {
    pub fn get(&self, entity: Entity) -> Result<&Model> {
        let (npc_id, transform) = self.transforms.get(entity)?;
        let region_id = RegionId::from(transform.translation);
        let Some(region_entity) = self.world_map.get(&region_id).copied() else {
            return Err(BevyError::from(format!(
                "Entity {entity:?} is not in any region"
            )));
        };

        let handles_table = self.regions.get(region_entity)?;
        handles_table.get_data(npc_id.as_ref(), &self.assets)
    }
}

#[derive(Component, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct RegionalNpcHandles(HashMap<Id, Handle<NpcInfo>>);
impl RegionalNpcHandles {
    pub fn get_data<'a>(
        &self,
        id: &Id,
        npc_info: &'a Assets<NpcInfo>,
    ) -> Result<&'a Model, BevyError> {
        let npc_handle = self.get(id).ok_or_else(|| {
            BevyError::from(format!("NPC with ID {} not found in data table", *id))
        })?;
        let npc_asset = npc_info.get(npc_handle).ok_or_else(|| {
            BevyError::from(format!("NPC asset not found for handle: {:?}", *npc_handle))
        })?;
        npc_asset
            .get(id)
            .ok_or_else(|| BevyError::from(format!("NPC model not found for ID: {}", *id)))
    }
}

#[derive(Debug, Event)]
pub struct Spawned;

#[derive(Debug, Event)]
pub struct Spawn {
    pub id: Id,
    pub transform: Transform,
}
