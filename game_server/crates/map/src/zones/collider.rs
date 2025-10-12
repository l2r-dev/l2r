use crate::{RegionGeoData, WorldMap};
use avian3d::prelude::*;
use bevy::{log, prelude::*};
use rand::{self, Rng};

pub trait RandomPointGenerator {
    fn generate_random_point_inside(&self, transform: &Transform) -> Vec3;
}

pub trait RandomPointWithGeodata {
    fn generate_random_point_with_geo(
        &self,
        transform: &Transform,
        geodata: &RegionGeoData,
    ) -> Vec3;
}

impl RandomPointGenerator for Collider {
    fn generate_random_point_inside(&self, transform: &Transform) -> Vec3 {
        let mut rng = rand::thread_rng();
        const MAX_ATTEMPTS: usize = 100;

        let aabb = self.aabb(transform.translation, transform.rotation);

        for _ in 0..MAX_ATTEMPTS {
            let random_point = Vec3::new(
                rng.gen_range(aabb.min.x..aabb.max.x),
                rng.gen_range(aabb.min.y..aabb.max.y),
                rng.gen_range(aabb.min.z..aabb.max.z),
            );

            if self.contains_point(transform.translation, transform.rotation, random_point) {
                return random_point;
            }
        }
        Vec3::INFINITY
    }
}

impl RandomPointWithGeodata for Collider {
    fn generate_random_point_with_geo(
        &self,
        transform: &Transform,
        geodata: &RegionGeoData,
    ) -> Vec3 {
        let random_point_collider = self.generate_random_point_inside(transform);
        // let random_point = geodata.random_point_in_radius(GeoVec3::from(random_point_collider), 2);
        let random_point =
            geodata.random_point_in_radius(WorldMap::vec3_to_geo(random_point_collider), 2);
        random_point
            .map(WorldMap::geo_to_vec3)
            .unwrap_or_else(||
                {
                    log::warn!("Failed to find random point with geodata, falling back to random point inside collider");
                    random_point_collider
                }
            )
    }
}
