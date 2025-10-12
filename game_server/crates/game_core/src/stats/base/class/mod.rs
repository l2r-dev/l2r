use crate::stats::*;
use avian3d::prelude::*;
use serde::{Deserialize, Serialize};

mod id;
mod tree;

pub use id::*;
use spatial::GameVec3;
pub use tree::*;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BaseClassStats {
    pub primal_stats: PrimalStats,
    pub base_speed: MovementStats,
    pub breath: u32,
    pub atk_range: u32,
    pub born_points: Vec<GameVec3>,
    pub collision_male: Option<ColliderInfo>,
    pub collision_female: Option<ColliderInfo>,
}
impl BaseClassStats {
    pub fn collider(&self, gender: Gender) -> Collider {
        match gender {
            Gender::Male => Collider::capsule(
                self.collision_male
                    .unwrap_or(self.collision_female.unwrap_or_default())
                    .radius,
                self.collision_male
                    .unwrap_or(self.collision_female.unwrap_or_default())
                    .height,
            ),
            Gender::Female => Collider::capsule(
                self.collision_female
                    .unwrap_or(self.collision_male.unwrap_or_default())
                    .radius,
                self.collision_female
                    .unwrap_or(self.collision_male.unwrap_or_default())
                    .height,
            ),
            _ => Collider::default(),
        }
    }
}
