use bevy::prelude::*;
use bitflags::bitflags;
use std::fmt;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct NavigationDirection: u8 {
        const EAST = 1;
        const WEST = 1 << 1;
        const SOUTH = 1 << 2;
        const NORTH = 1 << 3;
        const NORTH_EAST = Self::NORTH.bits() | Self::EAST.bits();
        const NORTH_WEST = Self::NORTH.bits() | Self::WEST.bits();
        const SOUTH_EAST = Self::SOUTH.bits() | Self::EAST.bits();
        const SOUTH_WEST = Self::SOUTH.bits() | Self::WEST.bits();
    }
}

impl NavigationDirection {}

/// Iterator that yields all directions contained in a NavigationDirection value,
/// including composite diagonal directions when both component directions are present.
/// to use simple directions - use NavigationDirection::iter()
pub struct CompositeDirectionsIter {
    remaining: u8,
    index: u8,
}

impl CompositeDirectionsIter {
    #[inline]
    fn new(flags: NavigationDirection) -> Self {
        Self {
            remaining: flags.bits(),
            index: 0,
        }
    }
}

impl Iterator for CompositeDirectionsIter {
    type Item = NavigationDirection;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        while self.index < NavigationDirection::ALL.len() as u8 {
            let direction = NavigationDirection::ALL[self.index as usize];
            self.index += 1;

            let bits = direction.bits();
            // Check if all bits of this direction are set in remaining
            if (self.remaining & bits) == bits {
                return Some(direction);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Count remaining possible directions based on set bits
        // Maximum is the number of directions we haven't checked yet
        let upper = (NavigationDirection::ALL.len() as u8 - self.index) as usize;
        (0, Some(upper))
    }
}

impl NavigationDirection {
    const COMPASS_LINES: [&'static str; 8] = [
        " SW  W  NW [N] NE  E  SE ",
        " W  NW  N [NE]  E  SE  S ",
        " NW  N  NE [E] SE  S  SW ",
        " N  NE  E [SE]  S  SW  W ",
        " NE  E  SE [S] SW  W  NW ",
        " E  SE  S [SW]  W  NW  N ",
        " SE  S  SW [W] NW  N  NE ",
        " S  SW  W [NW]  N  NE  E ",
    ];

    const ALL: [NavigationDirection; 8] = [
        NavigationDirection::NORTH,
        NavigationDirection::NORTH_EAST,
        NavigationDirection::EAST,
        NavigationDirection::SOUTH_EAST,
        NavigationDirection::SOUTH,
        NavigationDirection::SOUTH_WEST,
        NavigationDirection::WEST,
        NavigationDirection::NORTH_WEST,
    ];

    #[inline]
    pub const fn index(&self) -> usize {
        match *self {
            NavigationDirection::NORTH => 0,
            NavigationDirection::NORTH_EAST => 1,
            NavigationDirection::EAST => 2,
            NavigationDirection::SOUTH_EAST => 3,
            NavigationDirection::SOUTH => 4,
            NavigationDirection::SOUTH_WEST => 5,
            NavigationDirection::WEST => 6,
            NavigationDirection::NORTH_WEST => 7,
            _ => 0,
        }
    }

    #[inline]
    pub fn compass_line(&self) -> &'static str {
        Self::COMPASS_LINES[self.index()]
    }

    /// Returns an iterator over all directions contained in this flags value.
    /// This includes composite diagonal directions when both component directions are present.
    ///
    /// # Example
    /// ```
    /// use spatial::NavigationDirection;
    /// let flags = NavigationDirection::EAST | NavigationDirection::NORTH;
    /// let dirs: Vec<_> = flags.iter_composite().collect();
    /// assert_eq!(dirs, [NavigationDirection::NORTH, NavigationDirection::NORTH_EAST, NavigationDirection::EAST]);
    /// ```
    #[inline]
    pub fn iter_composite(self) -> CompositeDirectionsIter {
        CompositeDirectionsIter::new(self)
    }

    /// Returns true if this is a diagonal direction.
    #[inline]
    pub fn is_diagonal(self) -> bool {
        matches!(
            self,
            NavigationDirection::NORTH_EAST
                | NavigationDirection::NORTH_WEST
                | NavigationDirection::SOUTH_EAST
                | NavigationDirection::SOUTH_WEST
        )
    }

    /// Decomposes a diagonal direction into its two straight components.
    /// Returns None for straight directions or invalid flags.
    #[inline]
    pub fn to_straight_directions(self) -> Option<[NavigationDirection; 2]> {
        match self {
            NavigationDirection::NORTH_EAST => {
                Some([NavigationDirection::NORTH, NavigationDirection::EAST])
            }
            NavigationDirection::NORTH_WEST => {
                Some([NavigationDirection::NORTH, NavigationDirection::WEST])
            }
            NavigationDirection::SOUTH_EAST => {
                Some([NavigationDirection::SOUTH, NavigationDirection::EAST])
            }
            NavigationDirection::SOUTH_WEST => {
                Some([NavigationDirection::SOUTH, NavigationDirection::WEST])
            }
            _ => None,
        }
    }

    /// Returns the (dx, dy) offset for this direction with the given step size.
    #[inline]
    pub fn offset(self, step: i32) -> (i32, i32) {
        match self {
            NavigationDirection::NORTH => (0, -step),
            NavigationDirection::SOUTH => (0, step),
            NavigationDirection::EAST => (step, 0),
            NavigationDirection::WEST => (-step, 0),
            NavigationDirection::NORTH_EAST => (step, -step),
            NavigationDirection::NORTH_WEST => (-step, -step),
            NavigationDirection::SOUTH_EAST => (step, step),
            NavigationDirection::SOUTH_WEST => (-step, step),
            _ => (0, 0),
        }
    }

    /// Creates a NavigationDirection from a (dx, dy) offset by taking the signum of each component.
    #[inline]
    pub fn from_offset(dx: i32, dy: i32) -> Self {
        match (dx.signum(), dy.signum()) {
            (1, 0) => NavigationDirection::EAST,
            (-1, 0) => NavigationDirection::WEST,
            (0, 1) => NavigationDirection::SOUTH,
            (0, -1) => NavigationDirection::NORTH,
            (1, 1) => NavigationDirection::SOUTH_EAST,
            (1, -1) => NavigationDirection::NORTH_EAST,
            (-1, 1) => NavigationDirection::SOUTH_WEST,
            (-1, -1) => NavigationDirection::NORTH_WEST,
            _ => NavigationDirection::empty(),
        }
    }
}

impl fmt::Display for NavigationDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "NONE");
        }
        if self.contains(NavigationDirection::NORTH) {
            f.write_str("N")?;
        }
        if self.contains(NavigationDirection::SOUTH) {
            f.write_str("S")?;
        }
        if self.contains(NavigationDirection::EAST) {
            f.write_str("E")?;
        }
        if self.contains(NavigationDirection::WEST) {
            f.write_str("W")?;
        }
        Ok(())
    }
}

impl From<Quat> for NavigationDirection {
    #[inline]
    fn from(quat: Quat) -> Self {
        let (yaw, _pitch, _roll) = quat.to_euler(EulerRot::YXZ);
        let mut yaw_degrees = yaw.to_degrees();

        // Normalize to [0, 360] range
        if yaw_degrees < 0.0 {
            yaw_degrees += 360.0;
        }

        match yaw_degrees {
            d if d < 22.5 => NavigationDirection::NORTH,
            22.5..67.5 => NavigationDirection::NORTH_WEST,
            67.5..112.5 => NavigationDirection::WEST,
            112.5..157.5 => NavigationDirection::SOUTH_WEST,
            157.5..202.5 => NavigationDirection::SOUTH,
            202.5..247.5 => NavigationDirection::SOUTH_EAST,
            247.5..292.5 => NavigationDirection::EAST,
            292.5..337.5 => NavigationDirection::NORTH_EAST,
            _ => NavigationDirection::NORTH, // >= 337.5
        }
    }
}

impl From<NavigationDirection> for Quat {
    fn from(flags: NavigationDirection) -> Self {
        let yaw_degrees: f32 = match flags {
            NavigationDirection::NORTH => 0.0,
            NavigationDirection::NORTH_WEST => 45.0,
            NavigationDirection::WEST => 90.0,
            NavigationDirection::SOUTH_WEST => 135.0,
            NavigationDirection::SOUTH => 180.0,
            NavigationDirection::SOUTH_EAST => 225.0,
            NavigationDirection::EAST => 270.0,
            NavigationDirection::NORTH_EAST => 315.0,
            _ => 0.0, // Default to North for empty or combined flags
        };
        Quat::from_euler(EulerRot::YXZ, yaw_degrees.to_radians(), 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_directions_iter() {
        let flags = NavigationDirection::EAST | NavigationDirection::NORTH;
        let collected: Vec<_> = flags.iter().collect();
        assert_eq!(collected.len(), 2);
        assert!(collected.contains(&NavigationDirection::EAST));
        assert!(collected.contains(&NavigationDirection::NORTH));

        let all = NavigationDirection::all();
        let collected: Vec<_> = all.iter().collect();
        assert_eq!(collected.len(), 4);
    }

    #[test]
    fn test_composite_directions_iter() {
        let flags = NavigationDirection::EAST | NavigationDirection::NORTH;
        let collected: Vec<_> = flags.iter_composite().collect();
        assert_eq!(collected.len(), 3);
        assert!(collected.contains(&NavigationDirection::EAST));
        assert!(collected.contains(&NavigationDirection::NORTH));
        assert!(collected.contains(&NavigationDirection::NORTH_EAST));

        let single = NavigationDirection::EAST;
        let collected: Vec<_> = single.iter_composite().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], NavigationDirection::EAST);

        let all = NavigationDirection::all();
        let collected: Vec<_> = all.iter_composite().collect();
        assert_eq!(collected.len(), 8);
    }

    #[test]
    fn test_navigation_direction_to_quat() {
        for direction in NavigationDirection::all().iter_composite() {
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
            (0.0, NavigationDirection::NORTH),
            (45.0, NavigationDirection::NORTH_WEST),
            (90.0, NavigationDirection::WEST),
            (135.0, NavigationDirection::SOUTH_WEST),
            (180.0, NavigationDirection::SOUTH),
            (225.0, NavigationDirection::SOUTH_EAST),
            (270.0, NavigationDirection::EAST),
            (315.0, NavigationDirection::NORTH_EAST),
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

        for direction in NavigationDirection::all().iter_composite() {
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
            (NavigationDirection::NORTH, (0, -step)),
            (NavigationDirection::SOUTH, (0, step)),
            (NavigationDirection::EAST, (step, 0)),
            (NavigationDirection::WEST, (-step, 0)),
            (NavigationDirection::NORTH_EAST, (step, -step)),
            (NavigationDirection::NORTH_WEST, (-step, -step)),
            (NavigationDirection::SOUTH_EAST, (step, step)),
            (NavigationDirection::SOUTH_WEST, (-step, step)),
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
            ((1, 0), NavigationDirection::EAST),
            ((-1, 0), NavigationDirection::WEST),
            ((0, 1), NavigationDirection::SOUTH),
            ((0, -1), NavigationDirection::NORTH),
            ((1, 1), NavigationDirection::SOUTH_EAST),
            ((1, -1), NavigationDirection::NORTH_EAST),
            ((-1, 1), NavigationDirection::SOUTH_WEST),
            ((-1, -1), NavigationDirection::NORTH_WEST),
            // Additional test cases with larger offsets
            ((3, 0), NavigationDirection::EAST),
            ((-5, 0), NavigationDirection::WEST),
            ((0, 7), NavigationDirection::SOUTH),
            ((0, -2), NavigationDirection::NORTH),
            ((2, 2), NavigationDirection::SOUTH_EAST),
            ((-3, -3), NavigationDirection::NORTH_WEST),
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
            for direction in NavigationDirection::all().iter_composite() {
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
        const DIAGONAL: [NavigationDirection; 4] = [
            NavigationDirection::NORTH_EAST,
            NavigationDirection::NORTH_WEST,
            NavigationDirection::SOUTH_EAST,
            NavigationDirection::SOUTH_WEST,
        ];

        const STRAIGHT: [NavigationDirection; 4] = [
            NavigationDirection::NORTH,
            NavigationDirection::SOUTH,
            NavigationDirection::EAST,
            NavigationDirection::WEST,
        ];

        for direction in DIAGONAL {
            assert!(
                direction.is_diagonal(),
                "Direction {:?} should be diagonal",
                direction
            );
        }

        for direction in STRAIGHT {
            assert!(
                !direction.is_diagonal(),
                "Direction {:?} should not be diagonal",
                direction
            );
        }

        for direction in NavigationDirection::all().iter_composite() {
            let is_diagonal = direction.is_diagonal();
            let expected = DIAGONAL.contains(&direction);
            assert_eq!(
                is_diagonal, expected,
                "Diagonal check failed for direction {:?}: expected {}, got {}",
                direction, expected, is_diagonal
            );
        }
    }

    #[test]
    fn test_compass_line() {
        let expected_lines = vec![
            (NavigationDirection::NORTH, " SW  W  NW [N] NE  E  SE "),
            (NavigationDirection::NORTH_EAST, " W  NW  N [NE]  E  SE  S "),
            (NavigationDirection::EAST, " NW  N  NE [E] SE  S  SW "),
            (NavigationDirection::SOUTH_EAST, " N  NE  E [SE]  S  SW  W "),
            (NavigationDirection::SOUTH, " NE  E  SE [S] SW  W  NW "),
            (NavigationDirection::SOUTH_WEST, " E  SE  S [SW]  W  NW  N "),
            (NavigationDirection::WEST, " SE  S  SW [W] NW  N  NE "),
            (NavigationDirection::NORTH_WEST, " S  SW  W [NW]  N  NE  E "),
        ];

        for (direction, expected_line) in expected_lines {
            let compass_line = direction.compass_line();
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
            NavigationDirection::EAST,
            NavigationDirection::WEST,
            NavigationDirection::SOUTH,
            NavigationDirection::NORTH,
            NavigationDirection::NORTH_EAST,
            NavigationDirection::NORTH_WEST,
            NavigationDirection::SOUTH_EAST,
            NavigationDirection::SOUTH_WEST,
        ] {
            let flags = NavigationDirection::from(direction);
            let derived_direction = NavigationDirection::from(flags);
            assert_eq!(
                direction, derived_direction,
                "Conversion between direction and flags failed for {:?}",
                direction
            );
        }
    }

    #[test]
    fn test_index() {
        let test_cases = vec![
            (NavigationDirection::NORTH, 0),
            (NavigationDirection::NORTH_EAST, 1),
            (NavigationDirection::EAST, 2),
            (NavigationDirection::SOUTH_EAST, 3),
            (NavigationDirection::SOUTH, 4),
            (NavigationDirection::SOUTH_WEST, 5),
            (NavigationDirection::WEST, 6),
            (NavigationDirection::NORTH_WEST, 7),
            (NavigationDirection::empty(), 0),
            (NavigationDirection::all(), 0),
        ];

        for (direction, expected_index) in test_cases {
            let index = direction.index();
            assert_eq!(
                index, expected_index,
                "Index mismatch for direction {:?}: expected {:?}, got {:?}",
                direction, expected_index, index
            );
        }

        for (expected_idx, direction) in NavigationDirection::all().iter_composite().enumerate() {
            let idx = direction.index();
            assert_eq!(
                idx, expected_idx,
                "index for {:?} should be {} but got {}",
                direction, expected_idx, idx
            );
        }
    }
}
