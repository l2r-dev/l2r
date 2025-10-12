use avian3d::{parry::shape::Capsule, prelude::Collider};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ColliderInfo {
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub height: f32,
}
impl Default for ColliderInfo {
    fn default() -> Self {
        Self {
            radius: 1.0,
            height: 1.0,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum ColliderSize {
    #[default]
    Normal,
    Grown,
}

pub trait ColliderCapsuleSize {
    fn radius(&self) -> f64;
    fn height(&self) -> f64;
    fn radius_f32(&self) -> f32;
    fn height_f32(&self) -> f32;
}

impl ColliderCapsuleSize for Collider {
    fn radius(&self) -> f64 {
        self.shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.radius)
            .unwrap_or(0.0) as f64
    }

    fn radius_f32(&self) -> f32 {
        self.shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.radius)
            .unwrap_or(0.0)
    }

    fn height(&self) -> f64 {
        self.shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.segment.length())
            .unwrap_or(0.0) as f64
    }

    fn height_f32(&self) -> f32 {
        self.shape()
            .as_shape::<Capsule>()
            .map(|shape| shape.segment.length())
            .unwrap_or(0.0)
    }
}
