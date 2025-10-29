use crate::{
    WorldMap, ZoneListHandle,
    block::{Block, BlockKind, Cell, GeoBlock},
    id::RegionId,
    info::{RegionInfo, RegionRespawnZone},
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use l2r_core::assets::binary::{BinaryAsset, BinaryAssetPlugin, BinaryLoaderError};
use spatial::{GameVec3, GeoPoint, GeoVec3, NavigationDirection};
use std::fmt;

pub mod block;
pub mod id;
pub mod info;

pub struct RegionComponentsPlugin;
impl Plugin for RegionComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BinaryAssetPlugin::<RegionGeoData>::new(&["l2j"]))
            .add_plugins(JsonAssetPlugin::<RegionInfo>::new(&["json"]));

        app.add_event::<LoadRegionItems>();

        app.register_type::<Region>()
            .register_type::<RegionRespawnZone>();
    }
}

#[derive(Clone, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Region {
    id: RegionId,
    handle: Handle<RegionGeoData>,
    zone_list_handle: ZoneListHandle,
    center_coordinates: Option<GameVec3>,
    #[reflect(ignore)]
    blocks_centers_coordinates: Option<Vec<GameVec3>>,
}

impl Region {
    pub const BLOCKS_X: i32 = 256;
    pub const BLOCKS_Y: i32 = 256;
    pub const BLOCKS: i32 = Self::BLOCKS_X * Self::BLOCKS_Y;
    pub const SIZE_X: i32 = Self::BLOCKS_X * Block::SIZE_X;
    pub const SIZE_Y: i32 = Self::BLOCKS_Y * Block::SIZE_Y;

    pub fn new(id: RegionId) -> Self {
        Self {
            id,
            handle: Handle::default(),
            center_coordinates: None,
            blocks_centers_coordinates: None,
            zone_list_handle: ZoneListHandle::default(),
        }
    }

    pub fn id(&self) -> RegionId {
        self.id
    }

    pub fn active(&self) -> bool {
        self.center_coordinates.is_some()
    }

    pub fn activate(&mut self, geodata: &RegionGeoData) {
        self.center_coordinates = Some(self.calculate_center_coordinates(geodata));
        self.blocks_centers_coordinates = Some(self.calculate_blocks_centers_coordinates(geodata));
    }

    pub fn handle(&self) -> &Handle<RegionGeoData> {
        &self.handle
    }

    pub fn set_handle(&mut self, handle: Handle<RegionGeoData>) {
        self.handle = handle;
    }

    pub fn zone_list_handle(&self) -> &ZoneListHandle {
        &self.zone_list_handle
    }

    pub fn set_zone_list_handle(&mut self, handle: ZoneListHandle) {
        self.zone_list_handle = handle;
    }

    pub fn center_coordinates(&self) -> Option<GameVec3> {
        self.center_coordinates
    }

    fn calculate_center_coordinates(&self, geodata: &RegionGeoData) -> GameVec3 {
        let region_x = self.id.x() as i32;
        let region_y = self.id.y() as i32;

        let center_x = ((region_x - WorldMap::REGION_WITH_ZERO_COORD_X) * Self::SIZE_X
            + (Self::SIZE_X / 2)
            - WorldMap::WORLD_X_MIN)
            / Cell::SIZE;
        let center_y = ((region_y - WorldMap::REGION_WITH_ZERO_COORD_Y) * Self::SIZE_Y
            + (Self::SIZE_Y / 2)
            - WorldMap::WORLD_Y_MIN)
            / Cell::SIZE;

        let mut geo_vec = GeoVec3 {
            point: GeoPoint::new(center_x, center_y),
            height: 0,
        };

        geo_vec.height = geodata.nearest_height(&geo_vec).unwrap_or(0);

        WorldMap::geo_to_game(geo_vec)
    }

    // Get left bottom corner of the region
    pub fn origin_coordinates(&self) -> GameVec3 {
        let region_x = self.id.x() as i32;
        let region_y = self.id.y() as i32;

        let origin_x = ((region_x - WorldMap::REGION_WITH_ZERO_COORD_X) * Self::SIZE_X
            - WorldMap::WORLD_X_MIN)
            >> 4;
        let origin_y = ((region_y - WorldMap::REGION_WITH_ZERO_COORD_Y) * Self::SIZE_Y
            - WorldMap::WORLD_Y_MIN)
            >> 4;

        let geo_vec = GeoVec3 {
            point: GeoPoint::new(origin_x, origin_y),
            height: 0,
        };
        WorldMap::geo_to_game(geo_vec)
    }

    pub fn block_centers(&self) -> Option<&Vec<GameVec3>> {
        self.blocks_centers_coordinates.as_ref()
    }

    pub fn calculate_blocks_centers_coordinates(&self, geodata: &RegionGeoData) -> Vec<GameVec3> {
        let mut centers = Vec::with_capacity(Self::BLOCKS as usize);

        for block_x in 0..Self::BLOCKS_X {
            for block_y in 0..Self::BLOCKS_Y {
                if let Some(block) = geodata.block_by_grid(block_x, block_y) {
                    let block_position = self.block_position_by_grid(block_x, block_y);
                    let mut block_center = GameVec3::new(
                        block_position.x + (Block::SIZE_X / 2),
                        block_position.y + (Block::SIZE_Y / 2),
                        block_position.z,
                    );
                    let block_center_geo = WorldMap::game_to_geo(block_center);

                    match block {
                        Block::Multilayer(multilayer_block) => {
                            let cells = multilayer_block
                                .layered_cell_by_loc(&block_center_geo)
                                .cells();

                            for cell in cells {
                                let mut block_center = GameVec3::new(
                                    block_position.x + (Block::SIZE_X / 2),
                                    block_position.y + (Block::SIZE_Y / 2),
                                    block_position.z,
                                );

                                block_center.z = cell.height();
                                centers.push(block_center);
                            }
                        }
                        Block::Complex(complex_block) => {
                            block_center.z = complex_block.nearest_height(&block_center_geo);
                            centers.push(block_center);
                        }
                        Block::Flat(flat_block) => {
                            block_center.z = flat_block.nearest_height(&block_center_geo);
                            centers.push(block_center);
                        }
                    }
                }
            }
        }
        centers
    }

    pub fn block_position_by_grid(&self, block_x: i32, block_y: i32) -> GameVec3 {
        let region_origin = self.origin_coordinates();
        let x = region_origin.x + (block_x * Block::SIZE_X);
        let y = region_origin.y + (block_y * Block::SIZE_Y);
        GameVec3::new(x, y, region_origin.z)
    }

    pub fn block_position_geo_by_grid(&self, block_x: i32, block_y: i32) -> GeoVec3 {
        let game_pos = self.block_position_by_grid(block_x, block_y);
        WorldMap::game_to_geo(game_pos)
    }
}

#[derive(Asset, Clone, Default, Resource, TypePath)]
pub struct RegionGeoData(Vec<Block>);
impl BinaryAsset for RegionGeoData {
    fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryLoaderError> {
        let mut blocks = Vec::with_capacity(Region::BLOCKS as usize);
        let mut i = 0;
        while i < bytes.len() {
            match Block::from_bytes(&bytes[i..]) {
                Ok(block) => {
                    i += block.size();
                    blocks.push(block);
                }
                Err(err) => return Err(err),
            }
        }
        Ok(RegionGeoData(blocks))
    }
}
impl fmt::Debug for RegionGeoData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegionGeoData")
            .field("blocks", &self.0.len())
            .finish()
    }
}
impl RegionGeoData {
    pub fn nearest_height(&self, loc: &GeoVec3) -> Option<i32> {
        Some(self.block_by_loc(loc)?.nearest_height(loc))
    }

    pub fn next_higher_height(&self, from: &GeoVec3, to: &GeoVec3) -> Option<i32> {
        Some(self.block_by_loc(from)?.next_higher_height(from, to))
    }

    pub fn can_move_to(&self, start: &GeoVec3, goal: &GeoVec3) -> bool {
        if start.point == goal.point {
            return true;
        }

        // If start and goal are in different regions, allow movement
        let start_region_id = RegionId::from(*start);
        let goal_region_id = RegionId::from(*goal);
        if start_region_id != goal_region_id {
            return true;
        }

        let mut prev_point = *start;
        for current_point in start.line_to(goal).skip(1) {
            if !self.can_step_to(&prev_point, &current_point) {
                return false;
            }
            if current_point == *goal {
                return true;
            }
            prev_point = current_point;
        }
        true
    }

    pub fn can_see_target(&self, start: GeoVec3, target: GeoVec3) -> bool {
        const ELEVATED_SEE_OVER_DISTANCE: usize = 3;
        const MAX_SEE_OVER_HEIGHT: i32 = 50;
        const MAX_VISIBILITY_DISTANCE: i32 = 1000;
        const MAX_VISIBILITY_DISTANCE_SQUARED: i32 =
            MAX_VISIBILITY_DISTANCE * MAX_VISIBILITY_DISTANCE;

        if start.point == target.point {
            return true;
        }

        if start.distance_squared(&target) > MAX_VISIBILITY_DISTANCE_SQUARED {
            return false;
        }

        let start_with_height = GeoVec3 {
            height: self.nearest_height(&start).unwrap_or(start.height),
            ..start
        };

        let target_with_height = GeoVec3 {
            height: self.nearest_height(&target).unwrap_or(target.height),
            ..target
        };

        // Always trace line of sight from the higher point to the lower point
        let (from, to) = if target_with_height.height > start_with_height.height {
            (target_with_height, start_with_height)
        } else {
            (start_with_height, target_with_height)
        };

        let mut prev_point = from;

        for (idx, current_point) in from.line_to(&to).skip(1).enumerate() {
            // Sometimes line_to returns the same point with different height, no need to check it
            if current_point.point == prev_point.point {
                continue;
            }

            // get_sight_height will handle missing geo data gracefully
            let current_sight_height = self.get_sight_height(&prev_point, &current_point);

            let max_sight_height = if idx < ELEVATED_SEE_OVER_DISTANCE {
                from.height + MAX_SEE_OVER_HEIGHT
            } else {
                current_point.height + MAX_SEE_OVER_HEIGHT
            };

            if current_sight_height > max_sight_height {
                return false;
            }

            prev_point = current_point;
        }

        true
    }

    pub fn random_point_in_radius_vec3(&self, center: Vec3, radius: i32) -> Option<Vec3> {
        self.random_point_in_radius(WorldMap::game_to_geo(center.into()), radius)
            .map(WorldMap::geo_to_vec3)
    }

    pub fn random_point_in_radius(&self, mut center: GeoVec3, radius: i32) -> Option<GeoVec3> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut iterations = 0;

        while iterations < 10 {
            let dx = rng.gen_range(-radius..=radius);
            let dy = rng.gen_range(-radius..=radius);

            let new_point = GeoPoint::new(center.point.x + dx, center.point.y + dy);

            center.height = self.nearest_height(&center).unwrap_or(center.height);

            let new_height = self.nearest_height(&GeoVec3::new(new_point, center.height));
            let new_height = match new_height {
                Some(height) => height,
                None => continue,
            };

            let new_loc = GeoVec3::new(new_point, new_height);

            let distance = new_point.distance(&center.point);

            let can_move = self.can_move_to(&center, &new_loc);

            if distance <= radius && can_move {
                return Some(new_loc);
            }

            iterations += 1;
        }

        None
    }

    pub fn block_kind(&self, loc: &GeoVec3) -> Option<BlockKind> {
        let block = self.block_by_loc(loc)?;
        Some(BlockKind::from(block))
    }

    fn block(&self, id: usize) -> Option<&Block> {
        self.0.get(id)
    }

    fn block_by_loc(&self, loc: &GeoVec3) -> Option<&Block> {
        let block_id = Block::id(loc);
        self.block(block_id)
    }

    pub fn block_by_grid(&self, block_x: i32, block_y: i32) -> Option<&Block> {
        let block_id = Block::id_grid(block_x, block_y).unwrap();
        self.block(block_id)
    }

    pub fn passable_directions(&self, loc: &GeoVec3) -> NavigationDirection {
        self.block_by_loc(loc)
            .map_or_else(NavigationDirection::empty, |block| {
                block.passable_directions(loc)
            })
    }

    fn is_passable_in(&self, loc: &GeoVec3, direction: NavigationDirection) -> bool {
        self.passable_directions(loc).contains(direction)
    }

    fn is_passable_to(&self, from: &GeoVec3, to: &GeoVec3) -> bool {
        if from.point == to.point {
            return true;
        }

        let direction_from_to = from.direction(to);

        if !self.is_passable_in(from, direction_from_to) {
            return false;
        }

        // For diagonal directions, check corner cutting
        if let Some([dir1, dir2]) = direction_from_to.to_straight_directions() {
            let neighbor1 = from.adjacent_position_in(dir1, None);
            let neighbor2 = from.adjacent_position_in(dir2, None);

            // Check if both straight directions are passable to the target
            let neigbor1_direction_to = neighbor1.direction(to);
            let neigbor2_direction_to = neighbor2.direction(to);

            if !self.is_passable_in(&neighbor1, neigbor1_direction_to)
                || !self.is_passable_in(&neighbor2, neigbor2_direction_to)
            {
                return false;
            }
        }

        true
    }

    pub fn can_step_to(&self, start: &GeoVec3, goal: &GeoVec3) -> bool {
        if start.point == goal.point {
            return true;
        }

        let start_height = self.nearest_height(start).unwrap_or(start.height);
        let goal_height = self.nearest_height(goal).unwrap_or(goal.height);

        // Only check height difference when going up
        if goal_height > start_height && (goal_height - start_height) > Cell::SIZE {
            return false;
        }

        self.is_passable_to(start, goal)
    }

    fn get_sight_height(&self, from: &GeoVec3, to: &GeoVec3) -> i32 {
        if self.is_passable_to(from, to) {
            self.nearest_height(from).unwrap_or(from.height)
        } else {
            self.next_higher_height(from, to).unwrap_or(to.height)
        }
    }
}

#[derive(Event)]
pub struct LoadRegionItems;
