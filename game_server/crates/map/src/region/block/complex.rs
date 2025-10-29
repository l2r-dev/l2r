use crate::block::{Block, Cell};
use bevy::prelude::*;
use l2r_core::assets::binary::BinaryLoaderError;
use spatial::{GeoPoint, GeoVec3, NavigationDirection};

#[derive(Clone, Debug, Reflect)]
pub struct ComplexBlock {
    cells: Vec<Cell>,
}

impl ComplexBlock {
    pub const TYPE: u8 = 1;
    const SIZE: usize = Block::CELLS as usize * 2 + 1;

    fn get_cell(&self, index: i32) -> &Cell {
        assert!(index < Block::CELLS, "Index out of bounds");
        self.cells.get(index as usize).unwrap()
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
}

impl Default for ComplexBlock {
    fn default() -> Self {
        Self {
            cells: vec![Cell::default(); Block::CELLS as usize],
        }
    }
}

impl super::GeoBlock for ComplexBlock {
    type Block = Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Block, BinaryLoaderError> {
        if bytes.len() < ComplexBlock::SIZE {
            return Err(BinaryLoaderError::BinaryParseError(
                "Insufficient bytes for ComplexBlock".to_string(),
            ));
        }

        let cells: Vec<Cell> = bytes[1..ComplexBlock::SIZE]
            .chunks_exact(2)
            .map(|chunk| Cell::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(ComplexBlock { cells })
    }

    fn size(&self) -> usize {
        Self::SIZE
    }

    fn cell_by_loc(&self, loc: &GeoVec3) -> &Cell {
        self.get_cell(Block::cell_offset(&GeoPoint::from(*loc)))
    }

    fn nearest_height(&self, loc: &GeoVec3) -> i32 {
        self.cell_by_loc(loc).height()
    }

    fn next_higher_height(&self, from: &GeoVec3, to: &GeoVec3) -> i32 {
        let cell_height = self.nearest_height(from);
        if cell_height >= to.height {
            cell_height
        } else {
            to.height
        }
    }

    fn passable_directions(&self, loc: &GeoVec3) -> NavigationDirection {
        self.cell_by_loc(loc).nswe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        block::Block,
        region::block::{Cell, GeoBlock},
    };
    use spatial::NavigationDirection;

    fn create_test_complex_block() -> ComplexBlock {
        let mut complex_block = ComplexBlock::default();
        for i in 0..Block::CELLS as usize {
            complex_block.cells[i] = Cell::new(i as i16);
        }
        complex_block
    }

    #[test]
    fn test_complexblock_new_empty() {
        let complex_block = ComplexBlock::default();
        assert_eq!(complex_block.cells.len(), Block::CELLS as usize);
        for cell in complex_block.cells {
            assert_eq!(cell, Cell::new(0));
        }
    }

    #[test]
    fn test_complexblock_new_insufficient_bytes() {
        let result = ComplexBlock::from_bytes(&[0x01; 10]);
        assert!(matches!(
            result,
            Err(BinaryLoaderError::BinaryParseError(_))
        ));
    }

    #[test]
    fn test_complexblock_new_sufficient_bytes() {
        let mut bytes = vec![ComplexBlock::TYPE];
        for i in 0..Block::CELLS as usize {
            bytes.extend((i as i16).to_le_bytes());
        }
        let result = ComplexBlock::from_bytes(&bytes);
        assert!(result.is_ok());
        let complex_block = result.unwrap();
        assert_eq!(complex_block.cells.len(), Block::CELLS as usize);
        for (i, cell) in complex_block.cells.iter().enumerate() {
            assert_eq!(cell.value(), i as i16);
        }
    }

    #[test]
    fn test_complexblock_get_cell() {
        let complex_block = create_test_complex_block();
        for i in 0..Block::CELLS {
            assert_eq!(complex_block.get_cell(i).value(), i as i16);
        }
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_complexblock_get_cell_out_of_bounds() {
        let complex_block = create_test_complex_block();
        complex_block.get_cell(Block::CELLS);
    }

    #[test]
    fn test_complexblock_get_cell_by_loc() {
        let complex_block = create_test_complex_block();
        let point = GeoPoint::new(5, 5);
        let loc = GeoVec3::new(point, 5);
        let cell_index = Block::cell_offset(&point);
        assert_eq!(complex_block.cell_by_loc(&loc).value(), cell_index as i16);
    }

    #[test]
    fn test_complexblock_get_nearest_height() {
        let mut complex_block = ComplexBlock::default();
        let point = GeoPoint::new(5, 5);
        let loc = GeoVec3::new(point, 5);
        let cell_index = Block::cell_offset(&point) as usize;
        complex_block.cells[cell_index] = Cell::new(1000);
        assert_eq!(complex_block.nearest_height(&loc), 496);
    }

    #[test]
    fn test_complexblock_get_nearest_nswe() {
        let mut complex_block = ComplexBlock::default();
        let point = GeoPoint::new(5, 5);
        let loc = GeoVec3::new(point, 5);
        let cell_index = Block::cell_offset(&point) as usize;
        complex_block.cells[cell_index] = Cell::new(0b1010); // NSWE flags
        assert_eq!(
            complex_block.passable_directions(&loc),
            NavigationDirection::from_bits_truncate(0b1010)
        );
    }
}
