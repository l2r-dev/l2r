use crate::id::RegionId;
use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};

mod constants;
mod doors;
mod region;
mod spawn_point;
mod zones;

pub use constants::*;
pub use doors::*;
pub use region::*;
use spatial::{GameVec3, GeoPoint, GeoVec3};
pub use spawn_point::*;
pub use zones::*;

pub struct WorldMapComponentsPlugin;

impl Plugin for WorldMapComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Region>()
            .register_type::<SpawnPoint>()
            .register_type::<Zone>()
            .register_type::<WorldMap>();

        app.init_resource::<WorldMap>();
    }
}

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
#[reflect(Resource)]
pub struct WorldMap(HashMap<RegionId, Entity>);

impl WorldMap {
    pub const REGION_X_MIN: u8 = REGION_X_MIN;
    pub const REGION_Y_MIN: u8 = REGION_Y_MIN;
    pub const REGION_X_MAX: u8 = REGION_X_MAX;
    pub const REGION_Y_MAX: u8 = REGION_Y_MAX;

    pub const REGION_X_SIZE: i16 = (Self::REGION_X_MAX - Self::REGION_X_MIN + 1) as i16;
    pub const REGION_Y_SIZE: i16 = (Self::REGION_Y_MAX - Self::REGION_Y_MIN + 1) as i16;

    pub const REGION_WITH_ZERO_COORD_X: i32 = 20;
    pub const REGION_WITH_ZERO_COORD_Y: i32 = 18;

    pub const WORLD_X_MIN: i32 =
        (Self::REGION_X_MIN as i32 - Self::REGION_WITH_ZERO_COORD_X) * Region::SIZE_X;
    pub const WORLD_Y_MIN: i32 =
        (Self::REGION_Y_MIN as i32 - Self::REGION_WITH_ZERO_COORD_Y) * Region::SIZE_Y;

    pub const WORLD_X_MAX: i32 =
        (Self::REGION_X_MAX as i32 - Self::REGION_WITH_ZERO_COORD_X + 1) * Region::SIZE_X;
    pub const WORLD_Y_MAX: i32 =
        (Self::REGION_Y_MAX as i32 - Self::REGION_WITH_ZERO_COORD_Y + 1) * Region::SIZE_Y;

    pub fn active_regions(&self) -> Vec<Entity> {
        self.iter()
            .map(|(_, entity)| *entity)
            .collect::<Vec<Entity>>()
    }

    pub fn active_regions_ids(&self) -> Vec<RegionId> {
        self.iter().map(|(id, _)| *id).collect::<Vec<RegionId>>()
    }

    pub fn region_geodata<'a>(
        &self,
        regions: &Query<Ref<Region>>,
        regions_geodata: &'a Res<Assets<RegionGeoData>>,
        region_id: RegionId,
    ) -> Result<&'a RegionGeoData, BevyError> {
        self.get(&region_id)
            .and_then(|entity| regions.get(*entity).ok())
            .and_then(|region| regions_geodata.get(region.handle().id()))
            .ok_or_else(|| BevyError::from("Failed to find geodata for region"))
    }

    /// Convert world coordinates to grid coordinates
    pub fn game_to_geo(value: GameVec3) -> GeoVec3 {
        // Convert world coordinates to grid coordinates using bit shifting
        // Subtract the world minimum to set the origin, then shift right by 4 bits to divide by 16
        // because each grid cell is 16 units wide
        let x = (value.x - Self::WORLD_X_MIN) >> 4;
        let y = (value.y - Self::WORLD_Y_MIN) >> 4;
        let z = value.z;
        GeoVec3::new(GeoPoint::new(x, y), z)
    }

    /// Convert grid coordinates back to world coordinates
    pub fn geo_to_game(value: GeoVec3) -> GameVec3 {
        GameVec3::new(
            // Convert grid coordinates back to world coordinates
            // Multiply by 16 to scale up to world units, then add world minimum to offset
            // Note: This conversion may not be exact if the original GameVec3 wasn't on the grid
            // adding Cell::SIZE / 2 to x and y to get the center of the cell
            (value.point.x << 4) + Self::WORLD_X_MIN + 8, // 8 is Cell::SIZE / 2
            (value.point.y << 4) + Self::WORLD_Y_MIN + 8, // 8 is Cell::SIZE / 2
            value.height,
        )
    }

    /// Convert Vec3 to GeoVec3 through GameVec3
    pub fn vec3_to_geo(value: Vec3) -> GeoVec3 {
        Self::game_to_geo(GameVec3::from(value))
    }

    /// Convert GeoVec3 to Vec3 through GameVec3
    pub fn geo_to_vec3(value: GeoVec3) -> Vec3 {
        Vec3::from(Self::geo_to_game(value))
    }
}

/// A SystemParam that encapsulates the common pattern of accessing world map data.
/// This combines the three frequently used together parameters: WorldMap, regions query, and region geodata assets.
#[derive(SystemParam)]
pub struct WorldMapQuery<'w, 's> {
    pub world_map: Res<'w, WorldMap>,
    pub regions: Query<'w, 's, Ref<'static, Region>>,
    pub regions_geodata: Res<'w, Assets<RegionGeoData>>,
}

impl<'w, 's> WorldMapQuery<'w, 's> {
    pub fn region_geodata(&self, region_id: RegionId) -> Result<&RegionGeoData, BevyError> {
        self.world_map
            .region_geodata(&self.regions, &self.regions_geodata, region_id)
    }

    pub fn region_geodata_from_pos(&self, position: Vec3) -> Result<&RegionGeoData, BevyError> {
        let region_id = RegionId::from(position);
        self.region_geodata(region_id)
    }

    pub fn region_geodata_from_game_pos(
        &self,
        position: GameVec3,
    ) -> Result<&RegionGeoData, BevyError> {
        let region_id = RegionId::from(position);
        self.region_geodata(region_id)
    }
}
