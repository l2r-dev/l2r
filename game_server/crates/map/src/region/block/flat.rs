use crate::block::Cell;
use l2r_core::assets::binary::BinaryLoaderError;
use spatial::{GeoVec3, NavigationDirection, NavigationFlags};

#[derive(Clone, Debug, Default)]
pub struct FlatBlock(Cell);

impl FlatBlock {
    pub const TYPE: u8 = 0;
    const SIZE: usize = 3;
}

impl super::GeoBlock for FlatBlock {
    type Block = Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Block, BinaryLoaderError> {
        if bytes.len() < Self::SIZE {
            return Err(BinaryLoaderError::BinaryParseError(
                "Insufficient bytes for FlatBlock".to_string(),
            ));
        }
        let cell = Cell::from_le_bytes([bytes[1], bytes[2]]);
        Ok(Self(cell))
    }

    fn size(&self) -> usize {
        Self::SIZE
    }

    fn cell_by_loc(&self, _: &GeoVec3) -> &Cell {
        &self.0
    }

    fn nearest_height(&self, _: &GeoVec3) -> i32 {
        // FlatBlock doesn't use the Cell::height() method,
        // because cells in such blocks store only the height without NSWE flags.
        self.0.value() as i32
    }

    fn next_higher_height(&self, from: &GeoVec3, to: &GeoVec3) -> i32 {
        let cell_height = self.nearest_height(from);
        if cell_height >= to.height {
            cell_height
        } else {
            to.height
        }
    }

    fn nearest_nswe(&self, _: &GeoVec3) -> NavigationFlags {
        // FlatBlock allows to go in all directions
        NavigationFlags::ALL
    }

    fn passable_directions(&self, _: &GeoVec3) -> Vec<NavigationDirection> {
        NavigationDirection::BASIC.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::block::GeoBlock;

    #[test]
    fn test_flatblock_new_insufficient_bytes() {
        let result = FlatBlock::from_bytes(&[0x00]);
        assert!(
            matches!(result, Err(BinaryLoaderError::BinaryParseError(_))),
            "Expected error for insufficient bytes"
        );
    }

    #[test]
    fn test_flatblock_get_nearest_nswe() {
        let cell = Cell::from_le_bytes([0x01, 0x0F]);
        let flat_block = FlatBlock(cell);
        assert_eq!(
            flat_block.nearest_nswe(&GeoVec3::default()),
            NavigationFlags::ALL,
            "The get_nearest_nswe() method should return all NSWE flags"
        );
    }

    #[test]
    fn test_flatblock_get_nearest_height() {
        let cell = Cell::from_le_bytes([0x01, 0x02]);
        let expected_height = 0x0201;
        let flat_block = FlatBlock(cell);
        assert_eq!(
            flat_block.nearest_height(&GeoVec3::default()),
            expected_height,
            "The get_nearest_height() method returns an incorrect height"
        );
    }
}
