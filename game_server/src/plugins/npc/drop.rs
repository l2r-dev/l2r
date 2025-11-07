use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use bevy_ecs::system::SystemParam;
use game_core::{
    active_action::ActiveAction,
    items::{ItemLocation, ItemsDataAccess, ItemsDataQueryMut, UniqueItem, model},
    network::{
        broadcast::{BroadcastScope, ServerPacketBroadcast},
        packets::server::{DropItem, GameServerPacket},
    },
    npc::{GenerateDropRequest, RegionalNpcInfoQuery},
    object_id::ObjectId,
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use map::{WorldMapQuery, id::RegionId};
use smallvec::SmallVec;

#[derive(SystemParam)]
pub struct DropSystemParams<'w, 's> {
    pub map_query: WorldMapQuery<'w, 's>,
    pub dropper_info:
        Query<'w, 's, (Ref<'static, Transform>, Ref<'static, ObjectId>), Without<ActiveAction>>,
    pub npc_info: RegionalNpcInfoQuery<'w, 's>,
    pub items_data: ItemsDataQueryMut<'w, 's>,
    pub repo_manager: Res<'w, RepositoryManager>,
}

pub struct GenerateDropPlugin;
impl Plugin for GenerateDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GenerateDropRequest>();

        app.add_observer(generate_drop_request_handler);
    }
}

pub fn generate_drop_request_handler(
    drop_request: Trigger<GenerateDropRequest>,
    mut commands: Commands,
    mut params: DropSystemParams,
) -> Result<()> {
    if params.repo_manager.is_mock() {
        return Ok(());
    }
    let dropper_entity = drop_request.target();
    let (dropper_transform, dropper_oid) = params.dropper_info.get(dropper_entity)?;
    let region_id = RegionId::from(dropper_transform.translation);

    let Some(region_entity) = params.map_query.inner.world_map.get(&region_id) else {
        return Err(BevyError::from("Failed to find region entity"));
    };
    let region = params.map_query.inner.regions.get(*region_entity)?;

    let Some(geodata) = params
        .map_query
        .inner
        .regions_geodata
        .get(region.handle().id())
    else {
        return Err(BevyError::from("Failed to find geodata for region"));
    };

    let npc_model = params.npc_info.get(dropper_entity)?;
    let Some(item_drops) = npc_model
        .drop_table
        .as_ref()
        .map(|drop_table| drop_table.calculate_drops())
    else {
        return Err(BevyError::from("Failed to calculate item drops"));
    };

    let mut spawned_items = SmallVec::<[model::Model; 8]>::new();

    for (drop_item_id, item_count) in item_drops {
        let new_object_id = params.items_data.object_id_manager.next_id();
        let Ok(item_info) = params.items_data.item_info(drop_item_id) else {
            return Err(BevyError::from("Failed to find item info"));
        };

        let Some(location) = geodata.random_point_in_radius_vec3(dropper_transform.translation, 3)
        else {
            return Err(BevyError::from("Failed to find drop location"));
        };

        let new_item = model::Model::new(
            new_object_id,
            drop_item_id,
            item_count,
            ItemLocation::World(location),
            None,
        );

        UniqueItem::from_model(new_item, item_info).spawn(&mut commands, item_info);

        let drop_item = DropItem::new(
            *dropper_oid,
            new_object_id,
            drop_item_id,
            location,
            item_info.stackable(),
            item_count,
        );

        commands.trigger_targets(
            ServerPacketBroadcast {
                packet: GameServerPacket::from(drop_item),
                scope: BroadcastScope::Known,
            },
            dropper_entity,
        );

        spawned_items.push(new_item);
    }

    let items_repository = params.repo_manager.typed::<ObjectId, model::Entity>()?;

    if !spawned_items.is_empty() {
        commands.spawn_task(move || async move {
            items_repository.create_many(spawned_items).await?;
            Ok(())
        });
    }
    Ok(())
}
