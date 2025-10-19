use bevy::prelude::*;

mod game_vec;
mod geo_point;
mod geo_vec;
mod heading;
mod navigation;

mod waypoint;

pub use game_vec::*;
pub use geo_point::*;
pub use geo_vec::*;
pub use heading::*;
pub use navigation::*;
pub use waypoint::*;

pub struct SpatialPlugin;
impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameVec3>()
            .register_type::<GeoPoint>()
            .register_type::<GeoVec3>()
            .register_type::<Heading>()
            .register_type::<WayPoint>();
    }
}

pub fn calculate_centroid(points: &[Vec3]) -> Option<Vec3> {
    let num_points = points.len();
    if num_points == 0 {
        return None; // Return None if the input slice is empty
    }
    if num_points == 1 {
        return Some(points[0]);
    }

    let sum = points
        .iter()
        .copied()
        .reduce(|a, b| a + b)
        .unwrap_or(Vec3::ZERO);
    Some(sum / num_points as f32)
}

pub trait FlatDistance {
    fn flat_distance(&self, other: &Self) -> f32;
}

impl FlatDistance for Vec3 {
    fn flat_distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
}

pub trait HeightDifference {
    const SIGNIFICANT_THRESHOLD: f32;
    fn height_diff(&self, other: &Self) -> f32;
    fn higher_than(&self, other: &Self) -> bool;
    fn lower_than(&self, other: &Self) -> bool;

    fn significant_higher_than(&self, other: &Self) -> bool {
        self.height_diff(other) > Self::SIGNIFICANT_THRESHOLD
    }
    fn significant_lower_than(&self, other: &Self) -> bool {
        self.height_diff(other) < -Self::SIGNIFICANT_THRESHOLD
    }
}

impl HeightDifference for Vec3 {
    const SIGNIFICANT_THRESHOLD: f32 = 50.0;

    fn height_diff(&self, other: &Self) -> f32 {
        self.y - other.y
    }

    fn higher_than(&self, other: &Self) -> bool {
        self.y > other.y
    }

    fn lower_than(&self, other: &Self) -> bool {
        self.y < other.y
    }
}

pub trait DirectionDegrees {
    fn direction_degrees(&self, other: &Self) -> f32;
}

impl DirectionDegrees for Vec3 {
    fn direction_degrees(&self, other: &Self) -> f32 {
        let direction_vec = *other - *self;
        let radians = f32::atan2(direction_vec.x, direction_vec.z);

        let degrees = radians.to_degrees() % 360.0;
        adjust_angle(degrees)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RelativeDirection {
    #[default]
    Face,
    Back,
    Side,
}

impl RelativeDirection {
    pub fn attack_bonus(&self) -> f32 {
        match self {
            RelativeDirection::Back => 1.2,
            RelativeDirection::Face => 1.0,
            RelativeDirection::Side => 1.1,
        }
    }
}

pub trait TransformRelativeDirection {
    fn relative_direction(&self, other: &Self) -> RelativeDirection;

    fn is_within_angle_relative_to(&self, other: &Self, angle: f32) -> bool;
}

impl TransformRelativeDirection for Transform {
    // This function calculates current entity direction to the target entity
    fn relative_direction(&self, other: &Self) -> RelativeDirection {
        let other_degrees = *Degrees::from(other.rotation);

        let direction_to_other = self.translation.direction_degrees(&other.translation);

        let mut angle_diff = (direction_to_other - other_degrees).abs() % 360.0;
        if angle_diff > 180.0 {
            angle_diff = 360.0 - angle_diff;
        };

        match angle_diff {
            angle if angle <= 60.0 => RelativeDirection::Face,
            angle if angle > 60.0 && angle < 120.0 => RelativeDirection::Side,
            _ => RelativeDirection::Back,
        }
    }

    fn is_within_angle_relative_to(&self, other: &Self, angle: f32) -> bool {
        let target_degrees = *Degrees::from(other.rotation);
        let direction_to_target = self.translation.direction_degrees(&other.translation);

        let mut angle_diff = (direction_to_target - target_degrees).abs() % 360.0;
        if angle_diff > 180.0 {
            angle_diff = 360.0 - angle_diff;
        }

        angle_diff <= (angle / 2.0)
    }
}
