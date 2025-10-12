use crate::block::{Block, Cell};
use bevy::prelude::*;
use l2r_core::assets::binary::BinaryLoaderError;
use spatial::{GeoPoint, GeoVec3, NavigationDirection, NavigationFlags};
use std::fmt;

/// A cell for [MultilayerBlock].
#[derive(Clone, Debug, Reflect)]
pub struct LayeredCell(Vec<Cell>);

impl LayeredCell {
    pub fn new(cells: Vec<Cell>) -> Self {
        LayeredCell(cells)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.0
    }

    pub fn nearest_height(&self, loc: &GeoVec3) -> i32 {
        self.nearest_cell(loc).height()
    }

    pub fn nearest_nswe(&self, loc: &GeoVec3) -> NavigationFlags {
        self.nearest_cell(loc).nswe()
    }

    fn nearest_cell(&self, loc: &GeoVec3) -> &Cell {
        self.0
            .iter()
            .min_by_key(|cell| (cell.height() - loc.height).abs())
            .unwrap()
    }
}

/// A block with multiple [Cell] in one [LayeredCell].
#[derive(Clone, Reflect)]
pub struct MultilayerBlock {
    pub cells: Vec<LayeredCell>,
}

impl fmt::Debug for MultilayerBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MultilayerBlock({} cells): {:?}",
            self.cells.len(),
            self.cells
        )
    }
}

impl MultilayerBlock {
    pub const TYPE: u8 = 2;

    pub fn layered_cell_by_loc(&self, loc: &GeoVec3) -> &LayeredCell {
        let index = Block::cell_offset(&GeoPoint::from(*loc));
        assert!(index < Block::CELLS, "Index out of bounds");
        self.cells.get(index as usize).unwrap()
    }
}

impl super::GeoBlock for MultilayerBlock {
    type Block = Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Block, BinaryLoaderError> {
        const MAX_LAYERS: usize = 125;
        let mut cells = Vec::with_capacity(Block::CELLS as usize);

        let mut offset = 1; // Skip the TYPE byte
        for _ in 0..Block::CELLS {
            // Check we have at least the layer count byte
            if offset >= bytes.len() {
                return Err(BinaryLoaderError::BinaryParseError(
                    "Geo file corrupted! Insufficient data.".to_string(),
                ));
            }

            let n_layers = bytes[offset] as usize;
            offset += 1;

            if n_layers == 0 || n_layers > MAX_LAYERS {
                return Err(BinaryLoaderError::BinaryParseError(
                    "Geo file corrupted! Invalid layers count!".to_string(),
                ));
            }

            let bytes_needed = n_layers * 2;
            if offset + bytes_needed > bytes.len() {
                return Err(BinaryLoaderError::BinaryParseError(
                    "Geo file corrupted! Insufficient data.".to_string(),
                ));
            }

            // Use array_chunks to process byte pairs directly without intermediate allocations
            let mut layered_cell = Vec::with_capacity(n_layers);
            for chunk in bytes[offset..offset + bytes_needed].chunks_exact(2) {
                layered_cell.push(Cell::from_le_bytes([chunk[0], chunk[1]]));
            }
            offset += bytes_needed;

            cells.push(LayeredCell::new(layered_cell));
        }

        if cells.len() != Block::CELLS as usize {
            return Err(BinaryLoaderError::BinaryParseError(
                "Geo file corrupted! Insufficient cells.".to_string(),
            ));
        }

        Ok(MultilayerBlock { cells })
    }

    fn size(&self) -> usize {
        self.cells
            .iter()
            .map(|layered_cell| layered_cell.0.len() * 2 + 1)
            .sum::<usize>()
            + 1 // TYPE byte
    }

    fn cell_by_loc(&self, loc: &GeoVec3) -> &Cell {
        self.layered_cell_by_loc(loc).nearest_cell(loc)
    }

    fn nearest_height(&self, loc: &GeoVec3) -> i32 {
        self.layered_cell_by_loc(loc).nearest_height(loc)
    }

    fn next_higher_height(&self, from: &GeoVec3, to: &GeoVec3) -> i32 {
        let cell = self.layered_cell_by_loc(from);

        let mut higher_height = i32::MAX;

        for layer in cell.cells() {
            let layer_height = layer.height();
            if layer_height == to.height {
                return layer_height;
            }

            if layer_height > to.height && layer_height < higher_height {
                higher_height = layer_height;
            }
        }

        if higher_height == i32::MAX {
            to.height
        } else {
            higher_height
        }
    }

    fn nearest_nswe(&self, loc: &GeoVec3) -> NavigationFlags {
        self.cell_by_loc(loc).nswe()
    }

    fn passable_directions(&self, loc: &GeoVec3) -> Vec<NavigationDirection> {
        self.cell_by_loc(loc).passable_directions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::block::GeoBlock;

    fn create_test_multilayer_block() -> MultilayerBlock {
        let mut cells = Vec::with_capacity(Block::CELLS as usize);
        for i in 0..Block::CELLS as i16 {
            let layered_cell = LayeredCell::new(vec![
                Cell::new(i - 3000 - 1000),
                Cell::new(i - 3000),
                Cell::new(i - 3000 + 1000),
            ]);
            cells.push(layered_cell);
        }
        MultilayerBlock { cells }
    }

    #[test]
    fn test_multilayer_block_new() {
        let mut bytes = vec![MultilayerBlock::TYPE];
        for _ in 0..Block::CELLS {
            bytes.push(3); // 3 layers
            bytes.extend_from_slice(&[0, 0, 1, 0, 2, 0]); // 3 cells
        }
        let result = MultilayerBlock::from_bytes(&bytes);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.cells.len(), Block::CELLS as usize);
        for layered_cell in &block.cells {
            assert_eq!(layered_cell.0.len(), 3);
        }
    }

    #[test]
    fn test_multilayer_block_new_insufficient_data() {
        let bytes = vec![MultilayerBlock::TYPE, 3]; // Insufficient data
        let result = MultilayerBlock::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(BinaryLoaderError::BinaryParseError(_))
        ));
    }

    #[test]
    fn test_multilayer_block_new_invalid_layers() {
        let mut bytes = vec![MultilayerBlock::TYPE];
        bytes.extend_from_slice(&[0]); // Invalid layer count
        let result = MultilayerBlock::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(BinaryLoaderError::BinaryParseError(_))
        ));
    }

    #[test]
    fn test_multilayer_block_len() {
        let block = create_test_multilayer_block();
        let expected_len = 1 + Block::CELLS as usize * (1 + 3 * 2); // TYPE + (layer count + 3 cells) for each cell
        assert_eq!(block.size(), expected_len);
    }

    #[test]
    fn test_multilayer_block_get_nearest_height() {
        let block = create_test_multilayer_block();
        let loc = GeoVec3::new(GeoPoint::new(1500, 3300), -1900);
        let z = block.nearest_height(&loc);
        assert_eq!(z, -1984);
    }

    #[test]
    fn test_multilayer_block_get_nearest_nswe() {
        let mut block = create_test_multilayer_block();
        let point = GeoPoint::new(5, 5);
        let loc = GeoVec3::new(point, 150);
        let cell_index = Block::cell_offset(&point) as usize;
        block.cells[cell_index] = LayeredCell::new(vec![Cell::new(0b1010)]);
        let nswe = block.nearest_nswe(&loc);
        assert_eq!(nswe, NavigationFlags::from_bits_truncate(0b1010));
    }

    #[test]
    fn test_layered_cell_nearest_cell() {
        let cells = vec![Cell::new(100), Cell::new(200), Cell::new(300)];
        let layered_cell = LayeredCell::new(cells);

        assert_eq!(
            layered_cell
                .nearest_cell(&GeoVec3::new(GeoPoint::new(0, 0), 60))
                .value(),
            100
        );
        assert_eq!(
            layered_cell
                .nearest_cell(&GeoVec3::new(GeoPoint::new(0, 0), 120))
                .value(),
            200
        );
        assert_eq!(
            layered_cell
                .nearest_cell(&GeoVec3::new(GeoPoint::new(0, 0), 180))
                .value(),
            300
        );
    }

    #[test]
    fn test_layered_cell_nearest_height() {
        let cells = vec![
            Cell::from_height(-3000),
            Cell::from_height(0),
            Cell::from_height(3000),
        ];
        let layered_cell = LayeredCell::new(cells);

        assert_eq!(
            layered_cell.nearest_height(&GeoVec3::new(GeoPoint::new(0, 0), -2950)),
            -3000
        );
        assert_eq!(
            layered_cell.nearest_height(&GeoVec3::new(GeoPoint::new(0, 0), -50)),
            0
        );
        assert_eq!(
            layered_cell.nearest_height(&GeoVec3::new(GeoPoint::new(0, 0), 3200)),
            3000
        );
    }

    #[test]
    fn test_layered_cell_nearest_nswe() {
        let cells = vec![
            Cell::from_height(-3000).set_nswe(NavigationFlags::NORTH),
            Cell::from_height(0).set_nswe(NavigationFlags::SOUTH),
            Cell::from_height(3000).set_nswe(NavigationFlags::ALL),
        ];
        let layered_cell = LayeredCell::new(cells);

        assert_eq!(
            layered_cell.nearest_nswe(&GeoVec3::new(GeoPoint::new(0, 0), -3000)),
            NavigationFlags::NORTH
        );
        assert_eq!(
            layered_cell.nearest_nswe(&GeoVec3::new(GeoPoint::new(0, 0), 0)),
            NavigationFlags::SOUTH
        );
        assert_eq!(
            layered_cell.nearest_nswe(&GeoVec3::new(GeoPoint::new(0, 0), 3000)),
            NavigationFlags::ALL
        );
    }
}
