mod cell;
mod complex;
mod flat;
mod multilayer;

use crate::Region;
use bevy::log;
pub use cell::*;
pub use complex::*;
use derive_more::From;
pub use flat::*;
use l2r_core::assets::binary::BinaryLoaderError;
pub use multilayer::*;
use spatial::{GeoPoint, GeoVec3, NavigationDirection};
use std::error::Error;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

pub trait GeoBlock {
    type Block;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Block, BinaryLoaderError>;
    fn size(&self) -> usize;
    fn cell_by_loc(&self, loc: GeoVec3) -> &Cell;
    fn nearest_height(&self, loc: GeoVec3) -> i32;
    fn next_higher_height(&self, from: GeoVec3, to: GeoVec3) -> i32;
    fn passable_directions(&self, loc: GeoVec3) -> NavigationDirection;
}

#[derive(Clone, EnumDiscriminants, From)]
#[strum_discriminants(name(BlockKind))]
#[strum_discriminants(derive(Display, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum Block {
    Flat(FlatBlock),
    Complex(ComplexBlock),
    Multilayer(MultilayerBlock),
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Flat(_) => write!(f, "Block::Flat"),
            Block::Complex(_) => write!(f, "Block::Complex"),
            Block::Multilayer(_) => write!(f, "Block::Multilayer"),
        }
    }
}

impl Block {
    pub const CELLS_X: i32 = 8;
    pub const CELLS_Y: i32 = 8;
    pub const CELLS: i32 = Self::CELLS_X * Self::CELLS_Y;
    pub const SIZE_X: i32 = Self::CELLS_X * Cell::SIZE;
    pub const SIZE_Y: i32 = Self::CELLS_Y * Cell::SIZE;

    pub fn id(loc: GeoVec3) -> usize {
        let block_id = ((loc.point.x / Block::CELLS_X) % Region::BLOCKS_X) * Region::BLOCKS_Y
            + (loc.point.y / Block::CELLS_Y) % Region::BLOCKS_Y;
        block_id as usize
    }

    // x and y in this case is a Block in 2D grid position in region
    pub fn id_grid(block_x: i32, block_y: i32) -> Result<usize, Box<dyn Error>> {
        if !(0..Region::BLOCKS_X).contains(&block_x) || !(0..Region::BLOCKS_Y).contains(&block_y) {
            log::error!(
                "Block coordinates out of bounds: block_x={}, block_y={}",
                block_x,
                block_y
            );
            return Err("Block coordinates out of bounds".into());
        }

        Ok((block_x * Region::BLOCKS_X + block_y) as usize)
    }

    pub fn grid_coordinates(loc: GeoVec3) -> (i32, i32) {
        let block_x = (loc.point.x / Block::CELLS_X) % Region::BLOCKS_X;
        let block_y = (loc.point.y / Block::CELLS_Y) % Region::BLOCKS_Y;
        (block_x, block_y)
    }

    pub fn cell_offset(loc: GeoPoint) -> i32 {
        (loc.x % Block::CELLS_X) * Block::CELLS_Y + (loc.y % Block::CELLS_Y)
    }
    pub fn cell_coordinates_by_offset(offset: i32) -> GeoPoint {
        let x = offset / Block::CELLS_Y;
        let y = offset % Block::CELLS_Y;
        GeoPoint::new(x, y)
    }

    /// Get all cells in a block with their local offsets.
    /// Returns a vector of (cell, offset) pairs.
    pub fn cells_with_offsets(&self) -> Vec<(&Cell, i32)> {
        match self {
            Block::Flat(block) => {
                // Flat block has only one cell for all positions
                vec![(block.cell(), 0)]
            }
            Block::Complex(block) => {
                // Complex block has a cell for each position
                block
                    .cells()
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| (cell, i as i32))
                    .collect()
            }
            Block::Multilayer(block) => {
                // Multilayer block can have multiple cells per position
                let mut result = Vec::new();
                for (offset, layered_cell) in block.layered_cells().iter().enumerate() {
                    for cell in layered_cell.cells() {
                        result.push((cell, offset as i32));
                    }
                }
                result
            }
        }
    }
}

impl GeoBlock for Block {
    type Block = Self;

    fn size(&self) -> usize {
        match self {
            Block::Flat(b) => b.size(),
            Block::Complex(b) => b.size(),
            Block::Multilayer(b) => b.size(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryLoaderError> {
        let block_type = bytes[0];
        match block_type {
            FlatBlock::TYPE => Ok(FlatBlock::from_bytes(bytes)?.into()),
            ComplexBlock::TYPE => Ok(ComplexBlock::from_bytes(bytes)?.into()),
            MultilayerBlock::TYPE => Ok(MultilayerBlock::from_bytes(bytes)?.into()),
            _ => Err(BinaryLoaderError::BinaryParseError(format!(
                "Invalid block type {block_type}"
            ))),
        }
    }

    fn cell_by_loc(&self, loc: GeoVec3) -> &Cell {
        match self {
            Block::Flat(block) => block.cell_by_loc(loc),
            Block::Complex(block) => block.cell_by_loc(loc),
            Block::Multilayer(block) => block.cell_by_loc(loc),
        }
    }

    fn nearest_height(&self, loc: GeoVec3) -> i32 {
        match self {
            Block::Flat(block) => block.nearest_height(loc),
            Block::Complex(block) => block.nearest_height(loc),
            Block::Multilayer(block) => block.nearest_height(loc),
        }
    }

    fn next_higher_height(&self, from: GeoVec3, to: GeoVec3) -> i32 {
        match self {
            Block::Flat(block) => block.next_higher_height(from, to),
            Block::Complex(block) => block.next_higher_height(from, to),
            Block::Multilayer(block) => block.next_higher_height(from, to),
        }
    }

    fn passable_directions(&self, loc: GeoVec3) -> NavigationDirection {
        match self {
            Block::Flat(block) => block.passable_directions(loc),
            Block::Complex(block) => block.passable_directions(loc),
            Block::Multilayer(block) => block.passable_directions(loc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spatial::GeoPoint;

    fn test_location_one() -> GeoPoint {
        GeoPoint::new(1, 3)
    }
    fn test_location_two() -> GeoPoint {
        GeoPoint::new(3, 7)
    }
    const TEST_OFFSET: i32 = 11;
    const TEST_OFFSET_TWO: i32 = 31;

    #[test]
    fn test_cell_offset() {
        assert_eq!(Block::cell_offset(&test_location_one()), TEST_OFFSET);
        assert_eq!(Block::cell_offset(&test_location_two()), TEST_OFFSET_TWO);
    }
    #[test]
    fn test_cell_coordinates_by_offset() {
        assert_eq!(
            Block::cell_coordinates_by_offset(TEST_OFFSET),
            test_location_one()
        );
        assert_eq!(
            Block::cell_coordinates_by_offset(TEST_OFFSET_TWO),
            test_location_two()
        );
    }

    #[test]
    fn test_offset_and_coordinates_consistency() {
        let point = GeoPoint::new(56, 10); // something bigger than 8x8
        let offset = Block::cell_offset(point);
        let recovered_loc = Block::cell_coordinates_by_offset(offset);
        assert_eq!(point.x % Block::CELLS_X, recovered_loc.x);
        assert_eq!(point.y % Block::CELLS_Y, recovered_loc.y);
    }
}
