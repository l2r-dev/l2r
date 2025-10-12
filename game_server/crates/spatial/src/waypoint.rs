use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct WayPoint {
    origin_location: Vec3,
    target_location: Vec3,
}

impl WayPoint {
    pub fn new(origin_location: Vec3, target_location: Vec3) -> Self {
        Self {
            origin_location,
            target_location,
        }
    }

    pub fn origin(&self) -> &Vec3 {
        &self.origin_location
    }

    pub fn target(&self) -> &Vec3 {
        &self.target_location
    }

    pub fn set_origin(&mut self, origin: Vec3) {
        self.origin_location = origin;
    }
}
