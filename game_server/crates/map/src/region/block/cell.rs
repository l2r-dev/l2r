use bevy::prelude::*;
use spatial::{NavigationDirection, NavigationFlags};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Reflect)]
pub struct Cell(i16);

impl Cell {
    pub const SIZE: i32 = 16;
    pub const HEIGHT_MASK: i16 = !0x000F; // All bits except the last 4

    pub fn new(data: i16) -> Self {
        Self(data)
    }

    pub fn value(&self) -> i16 {
        self.0
    }

    pub fn from_height(height: i32) -> Self {
        let height = height.clamp(i16::MIN as i32, i16::MAX as i32);
        let cell_value = ((height << 1) as i16) | NavigationFlags::ALL.bits() as i16;
        Cell::new(cell_value)
    }

    pub fn height(&self) -> i32 {
        ((self.0 & Self::HEIGHT_MASK) >> 1) as i32
    }

    pub fn nswe(&self) -> NavigationFlags {
        NavigationFlags::from_bits_truncate((self.0 & !Self::HEIGHT_MASK) as u8)
    }

    pub fn set_nswe(&mut self, nswe: NavigationFlags) -> Self {
        self.0 = (self.0 & Self::HEIGHT_MASK) | nswe.bits() as i16;
        *self
    }

    pub fn is_passable(&self, direction: NavigationDirection) -> bool {
        let nav_flags = NavigationFlags::from(direction);
        self.nswe().contains(nav_flags)
    }

    pub fn passable_directions(&self) -> Vec<NavigationDirection> {
        NavigationDirection::BASIC
            .into_iter()
            .filter(|dir| self.is_passable(*dir))
            .collect()
    }

    pub fn is_fully_blocked(&self) -> bool {
        self.nswe().is_empty()
    }

    pub fn from_le_bytes(bytes: [u8; 2]) -> Self {
        Self(i16::from_le_bytes(bytes))
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cell = Cell::new(0x1234);
        assert_eq!(cell.value(), 0x1234);
    }

    #[test]
    fn test_height() {
        let cell = Cell::new(0x1234);
        assert_eq!(cell.height(), 2328);
    }

    #[test]
    fn test_nswe() {
        let cell = Cell::new(0x1234);
        assert_eq!(cell.nswe(), NavigationFlags::from_bits_truncate(4));
    }

    #[test]
    fn test_set_nswe() {
        let mut cell = Cell::new(0x1230);
        cell.set_nswe(NavigationFlags::NORTH | NavigationFlags::SOUTH);
        assert_eq!(cell.value() & !Cell::HEIGHT_MASK, 0x0C);
    }

    #[test]
    fn test_is_passable() {
        let mut cell = Cell::new(0x1235);
        cell.set_nswe(NavigationFlags::NORTH | NavigationFlags::SOUTH);
        assert!(cell.is_passable(NavigationDirection::North));
        assert!(!cell.is_passable(NavigationDirection::East));
        assert!(cell.is_passable(NavigationDirection::South));
        assert!(!cell.is_passable(NavigationDirection::West));
    }

    #[test]
    fn test_from_height() {
        let test_cases = vec![(1000, 1000), (-1000, -1000), (5000, 5000), (-5000, -5000)];

        for (input_height, expected_height) in test_cases {
            let cell = Cell::from_height(input_height);
            assert_eq!(
                cell.height(),
                expected_height,
                "Height mismatch for input {}",
                input_height
            );
            assert_eq!(
                cell.nswe(),
                NavigationFlags::ALL,
                "NSWE flags should be ALL for input {}",
                input_height
            );
        }
    }

    #[test]
    fn test_from_le_bytes() {
        let bytes = [0x34, 0x12];
        let cell = Cell::from_le_bytes(bytes);
        assert_eq!(cell.value(), 0x1234);
    }

    #[test]
    fn test_to_le_bytes() {
        let cell = Cell::new(0x1234);
        assert_eq!(cell.to_le_bytes(), [0x34, 0x12]);
    }

    #[test]
    fn test_passable_directions() {
        let mut cell = Cell::new(0);
        cell.set_nswe(NavigationFlags::NORTH | NavigationFlags::EAST);
        let directions = cell.passable_directions();
        assert_eq!(directions.len(), 3);
        assert!(directions.contains(&NavigationDirection::North));
        assert!(directions.contains(&NavigationDirection::East));
        assert!(directions.contains(&NavigationDirection::NorthEast));
    }
}
