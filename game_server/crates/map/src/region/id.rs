use super::Region;
use crate::WorldMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use spatial::{GameVec3, GeoVec3, NavigationDirection};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub struct RegionId(u8, u8);
impl RegionId {
    pub fn new(x: u8, y: u8) -> Self {
        RegionId(x, y)
    }
    pub fn x(&self) -> u8 {
        self.0
    }
    pub fn y(&self) -> u8 {
        self.1
    }

    pub fn get_adjacent(&self, direction: NavigationDirection) -> RegionId {
        let (dx, dy) = direction.offset(1);

        let new_x =
            (self.x() as i16 - WorldMap::REGION_X_MIN as i16 + dx as i16 + WorldMap::REGION_X_SIZE)
                % WorldMap::REGION_X_SIZE
                + WorldMap::REGION_X_MIN as i16;
        let new_y =
            (self.y() as i16 - WorldMap::REGION_Y_MIN as i16 + dy as i16 + WorldMap::REGION_Y_SIZE)
                % WorldMap::REGION_Y_SIZE
                + WorldMap::REGION_Y_MIN as i16;

        RegionId(new_x as u8, new_y as u8)
    }
}

impl std::fmt::Display for RegionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.0, self.1)
    }
}

impl From<Vec3> for RegionId {
    fn from(value: Vec3) -> Self {
        RegionId::from(GameVec3::from(value))
    }
}

fn floor_div(a: i32, b: i32) -> i32 {
    assert!(b > 0, "Region size must be positive");
    let div = a / b;
    let rem = a % b;
    if rem != 0 && a < 0 { div - 1 } else { div }
}

impl From<GameVec3> for RegionId {
    fn from(value: GameVec3) -> Self {
        let region_x = floor_div(value.x, Region::SIZE_X) + WorldMap::REGION_WITH_ZERO_COORD_X;
        let region_y = floor_div(value.y, Region::SIZE_Y) + WorldMap::REGION_WITH_ZERO_COORD_Y;
        let clamped_x =
            region_x.clamp(WorldMap::REGION_X_MIN as i32, WorldMap::REGION_X_MAX as i32);
        let clamped_y =
            region_y.clamp(WorldMap::REGION_Y_MIN as i32, WorldMap::REGION_Y_MAX as i32);
        RegionId(clamped_x as u8, clamped_y as u8)
    }
}

impl From<GeoVec3> for RegionId {
    fn from(value: GeoVec3) -> Self {
        RegionId::from(WorldMap::geo_to_game(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_id_new() {
        let vec = GameVec3::new(28000, 10000, -4232);
        let region_id = RegionId::from(vec);
        assert_eq!(region_id.x(), 20);
        assert_eq!(region_id.y(), 18);
        let vec = GameVec3::new(-337, 7211, -3622);
        let region_id = RegionId::from(vec);
        assert_eq!(region_id.x(), 19);
        assert_eq!(region_id.y(), 18);
        let vec = GameVec3::new(-61486, 98344, -3715);
        let region_id = RegionId::from(vec);
        assert_eq!(region_id.x(), 18);
        assert_eq!(region_id.y(), 21);
    }

    #[test]
    fn test_region_id_to_string() {
        let region_id = RegionId::from(GameVec3::new(28302, 11008, -4232));
        assert_eq!(region_id.to_string(), "20_18");
        // GameVec3 { x: 57666, y: 114257, z: -2248 }
        let region_id = RegionId::from(GameVec3::new(57666, 114257, -2248));
        assert_eq!(region_id.to_string(), "21_21");
    }

    #[test]
    fn test_get_adjacent_simple_directions() {
        let center = RegionId(20, 18);

        assert_eq!(
            center.get_adjacent(NavigationDirection::North),
            RegionId(20, 17)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::South),
            RegionId(20, 19)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::East),
            RegionId(21, 18)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::West),
            RegionId(19, 18)
        );
    }

    #[test]
    fn test_get_adjacent_diagonal_directions() {
        let center = RegionId(20, 18);

        assert_eq!(
            center.get_adjacent(NavigationDirection::NorthEast),
            RegionId(21, 17)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::NorthWest),
            RegionId(19, 17)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::SouthEast),
            RegionId(21, 19)
        );
        assert_eq!(
            center.get_adjacent(NavigationDirection::SouthWest),
            RegionId(19, 19)
        );
    }

    #[test]
    fn test_get_adjacent_wrapping_x() {
        let east_edge = RegionId(WorldMap::REGION_X_MAX, 18);
        let west_edge = RegionId(WorldMap::REGION_X_MIN, 18);

        assert_eq!(
            east_edge.get_adjacent(NavigationDirection::East),
            RegionId(WorldMap::REGION_X_MIN, 18)
        );
        assert_eq!(
            west_edge.get_adjacent(NavigationDirection::West),
            RegionId(WorldMap::REGION_X_MAX, 18)
        );
    }

    #[test]
    fn test_get_adjacent_wrapping_y() {
        let north_edge = RegionId(20, WorldMap::REGION_Y_MIN);
        let south_edge = RegionId(20, WorldMap::REGION_Y_MAX);

        assert_eq!(
            north_edge.get_adjacent(NavigationDirection::North),
            RegionId(20, WorldMap::REGION_Y_MAX)
        );
        assert_eq!(
            south_edge.get_adjacent(NavigationDirection::South),
            RegionId(20, WorldMap::REGION_Y_MIN)
        );
    }

    #[test]
    fn test_get_adjacent_wrapping_corner() {
        let northeast_corner = RegionId(WorldMap::REGION_X_MAX, WorldMap::REGION_Y_MIN);
        let southwest_corner = RegionId(WorldMap::REGION_X_MIN, WorldMap::REGION_Y_MAX);

        assert_eq!(
            northeast_corner.get_adjacent(NavigationDirection::NorthEast),
            RegionId(WorldMap::REGION_X_MIN, WorldMap::REGION_Y_MAX)
        );
        assert_eq!(
            southwest_corner.get_adjacent(NavigationDirection::SouthWest),
            RegionId(WorldMap::REGION_X_MAX, WorldMap::REGION_Y_MIN)
        );
    }
}
