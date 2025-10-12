use bevy::prelude::*;
use bitflags::bitflags;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::fmt;
use strum::EnumIter;

bitflags! {
    #[derive(Debug, Default, PartialEq)]
    pub struct NavigationFlags: u8 {
        const EAST = 1 << 0;
        const WEST = 1 << 1;
        const SOUTH = 1 << 2;
        const NORTH = 1 << 3;
        const NORTHEAST = Self::NORTH.bits() | Self::EAST.bits();
        const NORTHWEST = Self::NORTH.bits() | Self::WEST.bits();
        const SOUTHEAST = Self::SOUTH.bits() | Self::EAST.bits();
        const SOUTHWEST = Self::SOUTH.bits() | Self::WEST.bits();
        const ALL = Self::EAST.bits() | Self::WEST.bits() | Self::SOUTH.bits() | Self::NORTH.bits();
    }
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq, FromPrimitive, IntoPrimitive, Reflect,
)]
pub enum NavigationDirection {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl fmt::Display for NavigationDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction_str = match self {
            NavigationDirection::North => "N",
            NavigationDirection::NorthEast => "NE",
            NavigationDirection::East => "E",
            NavigationDirection::SouthEast => "SE",
            NavigationDirection::South => "S",
            NavigationDirection::SouthWest => "SW",
            NavigationDirection::West => "W",
            NavigationDirection::NorthWest => "NW",
        };
        write!(f, "{direction_str}")
    }
}

impl NavigationDirection {
    pub const STRAIGHT: [NavigationDirection; 4] = [
        NavigationDirection::East,
        NavigationDirection::West,
        NavigationDirection::South,
        NavigationDirection::North,
    ];

    pub const DIAGONAL: [NavigationDirection; 4] = [
        NavigationDirection::NorthEast,
        NavigationDirection::NorthWest,
        NavigationDirection::SouthEast,
        NavigationDirection::SouthWest,
    ];
    pub const BASIC: [NavigationDirection; 8] = [
        NavigationDirection::North,
        NavigationDirection::NorthEast,
        NavigationDirection::East,
        NavigationDirection::SouthEast,
        NavigationDirection::South,
        NavigationDirection::SouthWest,
        NavigationDirection::West,
        NavigationDirection::NorthWest,
    ];

    pub fn perpendicular_directions(&self) -> Option<[NavigationDirection; 2]> {
        let index: u8 = (*self).into();
        Some([
            NavigationDirection::from_primitive((index + 2) % 8),
            NavigationDirection::from_primitive((index + 6) % 8),
        ])
    }

    pub fn offset(&self, step: i32) -> (i32, i32) {
        match self {
            NavigationDirection::North => (0, -step),
            NavigationDirection::South => (0, step),
            NavigationDirection::East => (step, 0),
            NavigationDirection::West => (-step, 0),
            NavigationDirection::NorthEast => (step, -step),
            NavigationDirection::NorthWest => (-step, -step),
            NavigationDirection::SouthEast => (step, step),
            NavigationDirection::SouthWest => (-step, step),
        }
    }

    pub fn from_offset(dx: i32, dy: i32) -> Self {
        match (dx.signum(), dy.signum()) {
            (1, 0) => NavigationDirection::East,
            (-1, 0) => NavigationDirection::West,
            (0, 1) => NavigationDirection::South,
            (0, -1) => NavigationDirection::North,
            (1, 1) => NavigationDirection::SouthEast,
            (1, -1) => NavigationDirection::NorthEast,
            (-1, 1) => NavigationDirection::SouthWest,
            (-1, -1) => NavigationDirection::NorthWest,
            _ => NavigationDirection::North,
        }
    }

    pub fn is_diagonal(&self) -> bool {
        NavigationDirection::DIAGONAL.contains(self)
    }

    pub fn opposite(&self) -> Self {
        match self {
            NavigationDirection::North => NavigationDirection::South,
            NavigationDirection::South => NavigationDirection::North,
            NavigationDirection::East => NavigationDirection::West,
            NavigationDirection::West => NavigationDirection::East,
            NavigationDirection::NorthEast => NavigationDirection::SouthWest,
            NavigationDirection::NorthWest => NavigationDirection::SouthEast,
            NavigationDirection::SouthEast => NavigationDirection::NorthWest,
            NavigationDirection::SouthWest => NavigationDirection::NorthEast,
        }
    }

    pub fn to_straight_directions(&self) -> Option<[NavigationDirection; 2]> {
        match self {
            NavigationDirection::NorthEast => {
                Some([NavigationDirection::North, NavigationDirection::East])
            }
            NavigationDirection::NorthWest => {
                Some([NavigationDirection::North, NavigationDirection::West])
            }
            NavigationDirection::SouthEast => {
                Some([NavigationDirection::South, NavigationDirection::East])
            }
            NavigationDirection::SouthWest => {
                Some([NavigationDirection::South, NavigationDirection::West])
            }
            _ => None,
        }
    }

    pub fn build_compass_line(&self) -> String {
        let current_index = NavigationDirection::BASIC
            .iter()
            .position(|&dir| dir == *self)
            .unwrap_or(0);

        let display_range = 3;
        let total_directions = NavigationDirection::BASIC.len();

        let mut compass_line = String::new();

        for offset in -display_range..=display_range {
            let index = (current_index as isize + offset + total_directions as isize)
                % total_directions as isize;
            let dir = NavigationDirection::BASIC[index as usize];

            if offset == 0 {
                compass_line.push_str(&format!("[{dir}]"));
            } else {
                compass_line.push_str(&format!(" {dir} "));
            }
        }

        compass_line
    }
}

impl From<NavigationDirection> for NavigationFlags {
    fn from(direction: NavigationDirection) -> Self {
        match direction {
            NavigationDirection::East => NavigationFlags::EAST,
            NavigationDirection::West => NavigationFlags::WEST,
            NavigationDirection::South => NavigationFlags::SOUTH,
            NavigationDirection::North => NavigationFlags::NORTH,
            NavigationDirection::NorthEast => NavigationFlags::NORTHEAST,
            NavigationDirection::NorthWest => NavigationFlags::NORTHWEST,
            NavigationDirection::SouthEast => NavigationFlags::SOUTHEAST,
            NavigationDirection::SouthWest => NavigationFlags::SOUTHWEST,
        }
    }
}

impl From<NavigationFlags> for NavigationDirection {
    fn from(flags: NavigationFlags) -> Self {
        match flags {
            NavigationFlags::EAST => NavigationDirection::East,
            NavigationFlags::WEST => NavigationDirection::West,
            NavigationFlags::SOUTH => NavigationDirection::South,
            NavigationFlags::NORTH => NavigationDirection::North,
            NavigationFlags::NORTHEAST => NavigationDirection::NorthEast,
            NavigationFlags::NORTHWEST => NavigationDirection::NorthWest,
            NavigationFlags::SOUTHEAST => NavigationDirection::SouthEast,
            NavigationFlags::SOUTHWEST => NavigationDirection::SouthWest,
            _ => NavigationDirection::North,
        }
    }
}

impl From<Quat> for NavigationDirection {
    fn from(quat: Quat) -> Self {
        let (yaw, _pitch, _roll) = quat.to_euler(EulerRot::YXZ);
        let mut yaw_degrees = yaw.to_degrees();
        if yaw_degrees < 0.0 {
            yaw_degrees += 360.0;
        }

        match yaw_degrees {
            x if !(22.5..337.5).contains(&x) => NavigationDirection::North,
            x if (22.5..67.5).contains(&x) => NavigationDirection::NorthWest,
            x if (67.5..112.5).contains(&x) => NavigationDirection::West,
            x if (112.5..157.5).contains(&x) => NavigationDirection::SouthWest,
            x if (157.5..202.5).contains(&x) => NavigationDirection::South,
            x if (202.5..247.5).contains(&x) => NavigationDirection::SouthEast,
            x if (247.5..292.5).contains(&x) => NavigationDirection::East,
            x if (292.5..337.5).contains(&x) => NavigationDirection::NorthEast,
            _ => NavigationDirection::North,
        }
    }
}

impl From<NavigationDirection> for Quat {
    fn from(direction: NavigationDirection) -> Self {
        let yaw_degrees: f32 = match direction {
            NavigationDirection::North => 0.0,
            NavigationDirection::NorthWest => 45.0,
            NavigationDirection::West => 90.0,
            NavigationDirection::SouthWest => 135.0,
            NavigationDirection::South => 180.0,
            NavigationDirection::SouthEast => 225.0,
            NavigationDirection::East => 270.0,
            NavigationDirection::NorthEast => 315.0,
        };
        Quat::from_euler(EulerRot::YXZ, yaw_degrees.to_radians(), 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_direction_to_quat() {
        for &direction in NavigationDirection::BASIC.iter() {
            let quat = Quat::from(direction);
            let direction_from_quat = NavigationDirection::from(quat);
            assert_eq!(
                direction, direction_from_quat,
                "Direction mismatch for: {:?}",
                direction
            );
        }
    }

    #[test]
    fn test_quat_to_navigation_direction() {
        let test_cases: Vec<(f32, NavigationDirection)> = vec![
            (0.0, NavigationDirection::North),
            (45.0, NavigationDirection::NorthWest),
            (90.0, NavigationDirection::West),
            (135.0, NavigationDirection::SouthWest),
            (180.0, NavigationDirection::South),
            (225.0, NavigationDirection::SouthEast),
            (270.0, NavigationDirection::East),
            (315.0, NavigationDirection::NorthEast),
        ];

        for (yaw_degrees, expected_direction) in test_cases {
            let quat = Quat::from_euler(EulerRot::YXZ, yaw_degrees.to_radians(), 0.0, 0.0);
            let direction = NavigationDirection::from(quat);
            assert_eq!(
                direction, expected_direction,
                "Direction mismatch for yaw: {} degrees",
                yaw_degrees
            );
        }
    }

    #[test]
    fn test_roundtrip_conversion() {
        let tolerance = 1e-4;

        for &direction in NavigationDirection::BASIC.iter() {
            let quat_from_direction: Quat = Quat::from(direction);
            let direction_from_quat = NavigationDirection::from(quat_from_direction);
            assert_eq!(
                direction, direction_from_quat,
                "Roundtrip conversion failed for: {:?}",
                direction
            );

            let quat_from_roundtrip = Quat::from(direction_from_quat);
            let diff = quat_from_direction.dot(quat_from_roundtrip);
            assert!(
                (1.0 - diff.abs()) < tolerance,
                "Quaternions are not equal: {:?} vs {:?}",
                quat_from_direction,
                quat_from_roundtrip
            );
        }
    }

    // New tests to verify the offset and from_offset methods
    #[test]
    fn test_offset_method() {
        let step = 1;
        let test_cases = vec![
            (NavigationDirection::North, (0, -step)),
            (NavigationDirection::South, (0, step)),
            (NavigationDirection::East, (step, 0)),
            (NavigationDirection::West, (-step, 0)),
            (NavigationDirection::NorthEast, (step, -step)),
            (NavigationDirection::NorthWest, (-step, -step)),
            (NavigationDirection::SouthEast, (step, step)),
            (NavigationDirection::SouthWest, (-step, step)),
        ];

        for (direction, expected_offset) in test_cases {
            let offset = direction.offset(step);
            assert_eq!(
                offset, expected_offset,
                "Offset mismatch for direction {:?}: expected {:?}, got {:?}",
                direction, expected_offset, offset
            );
        }
    }

    #[test]
    fn test_from_offset_method() {
        let test_cases = vec![
            ((1, 0), NavigationDirection::East),
            ((-1, 0), NavigationDirection::West),
            ((0, 1), NavigationDirection::South),
            ((0, -1), NavigationDirection::North),
            ((1, 1), NavigationDirection::SouthEast),
            ((1, -1), NavigationDirection::NorthEast),
            ((-1, 1), NavigationDirection::SouthWest),
            ((-1, -1), NavigationDirection::NorthWest),
            // Additional test cases with larger offsets
            ((3, 0), NavigationDirection::East),
            ((-5, 0), NavigationDirection::West),
            ((0, 7), NavigationDirection::South),
            ((0, -2), NavigationDirection::North),
            ((2, 2), NavigationDirection::SouthEast),
            ((-3, -3), NavigationDirection::NorthWest),
        ];

        for ((dx, dy), expected_direction) in test_cases {
            let direction = NavigationDirection::from_offset(dx, dy);
            assert_eq!(
                direction, expected_direction,
                "Direction mismatch for offset ({}, {}): expected {:?}, got {:?}",
                dx, dy, expected_direction, direction
            );
        }
    }

    #[test]
    fn test_offset_and_from_offset_roundtrip() {
        let steps = [1, 5, 10];
        for &step in &steps {
            for &direction in NavigationDirection::BASIC.iter() {
                let (dx, dy) = direction.offset(step);
                let derived_direction = NavigationDirection::from_offset(dx, dy);
                assert_eq!(
                    direction, derived_direction,
                    "Roundtrip failed for direction {:?} with step {}: offset ({}, {}), derived direction {:?}",
                    direction, step, dx, dy, derived_direction
                );
            }
        }
    }

    #[test]
    fn test_is_diagonal() {
        for &direction in NavigationDirection::BASIC.iter() {
            let is_diagonal = direction.is_diagonal();
            let expected = NavigationDirection::DIAGONAL.contains(&direction);
            assert_eq!(
                is_diagonal, expected,
                "Diagonal check failed for direction {:?}: expected {}, got {}",
                direction, expected, is_diagonal
            );
        }
    }

    #[test]
    fn test_build_compass_line() {
        let expected_lines = vec![
            (NavigationDirection::North, " SW  W  NW [N] NE  E  SE "),
            (NavigationDirection::South, " NE  E  SE [S] SW  W  NW "),
        ];

        for (direction, expected_line) in expected_lines {
            let compass_line = direction.build_compass_line();
            assert_eq!(
                compass_line, expected_line,
                "Compass line mismatch for direction {:?}: expected '{}', got '{}'",
                direction, expected_line, compass_line
            );
        }
    }

    #[test]
    fn test_to_flags_and_from_flags() {
        for &direction in &[
            NavigationDirection::East,
            NavigationDirection::West,
            NavigationDirection::South,
            NavigationDirection::North,
            NavigationDirection::NorthEast,
            NavigationDirection::NorthWest,
            NavigationDirection::SouthEast,
            NavigationDirection::SouthWest,
        ] {
            let flags = NavigationFlags::from(direction);
            let derived_direction = NavigationDirection::from(flags);
            assert_eq!(
                direction, derived_direction,
                "Conversion between direction and flags failed for {:?}",
                direction
            );
        }
    }
}
