use super::NavigationDirection;
use bevy::prelude::*;
use derive_more::{From, Into};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Deref, Deserialize, Eq, PartialEq, Serialize, Reflect, From, Into,
)]
pub struct Heading(i32);
impl Heading {
    pub const FULL_CIRCLE: f32 = 360.0;
    pub const HEADING_SCALE: f32 = (u16::MAX as f32 + 1.0) / Self::FULL_CIRCLE;

    pub fn new(heading: i32) -> Self {
        Heading(heading)
    }

    pub fn random() -> Self {
        let random = rand::thread_rng().gen_range(-(u16::MAX as i32)..(u16::MAX as i32));
        Heading(random)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deref,
    Deserialize,
    PartialEq,
    Serialize,
    Reflect,
    From,
    Into,
    PartialOrd,
)]
pub struct Degrees(f32);

impl Degrees {
    pub fn from_angle_between(vec1: Vec3, vec2: Vec3) -> Self {
        let angle_radians = vec1.angle_between(vec2);
        Degrees(angle_radians.to_degrees())
    }
}

impl PartialEq<f32> for Degrees {
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<f32> for Degrees {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

// Need substract from 270 because on server 0 is North, but in game heading 0 is East
#[inline]
pub fn adjust_angle(x: f32) -> f32 {
    (270.0 - x).rem_euclid(Heading::FULL_CIRCLE)
}

impl From<Quat> for Degrees {
    fn from(rotation: Quat) -> Self {
        let (yaw, _pitch, _roll) = rotation.to_euler(EulerRot::YXZ);
        let normalized_yaw = yaw.to_degrees();
        adjust_angle(normalized_yaw).into()
    }
}

impl From<Quat> for Heading {
    fn from(rotation: Quat) -> Self {
        let heading_degrees = *Degrees::from(rotation);
        Heading((heading_degrees * Heading::HEADING_SCALE) as i32)
    }
}

impl From<Heading> for Quat {
    fn from(heading: Heading) -> Self {
        let heading_degrees = *heading as f32 / Heading::HEADING_SCALE;
        let yaw_radians = adjust_angle(heading_degrees).to_radians();
        Quat::from_euler(EulerRot::YXZ, yaw_radians, 0.0, 0.0)
    }
}

impl From<NavigationDirection> for Heading {
    fn from(flags: NavigationDirection) -> Self {
        let quat = Quat::from(flags);
        Heading::from(quat)
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_flags() {
        let flags = NavigationDirection::EAST;
        let heading = Heading::from(flags);
        assert_eq!(0, *heading);

        let flags = NavigationDirection::NORTH;
        let heading = Heading::from(flags);
        assert_eq!(49152, *heading);

        let new_heading = Heading::new(49152);
        assert_eq!(heading, new_heading);

        let new_heading2 = Heading::new(-49152);
        let quat = Quat::from(new_heading2);
        let new_flags = NavigationDirection::from(quat);
        assert_eq!(NavigationDirection::SOUTH, new_flags);
    }
}
