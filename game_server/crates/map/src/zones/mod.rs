use avian3d::prelude::*;
use bevy::{
    ecs::{
        component::{ComponentHook, Mutable, StorageType},
        world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::{Deserialize, Serialize};

pub mod collider;
mod kind;

pub use kind::*;

pub struct ZonesComponentsPlugin;
impl Plugin for ZonesComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<ZoneList>::new(&["json"]));

        app.register_type::<Zone>()
            .register_type::<GlobalZones>()
            .register_type::<RegionalZones>()
            .register_type::<ZoneKindVariant>()
            .register_type::<NamedZones>();

        app.init_resource::<NamedZones>();
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(Name::new("Zones".to_string()))]
pub struct GlobalZones;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(Name::new("Zones".to_string()))]
pub struct RegionalZones;

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct ZonePoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub enum ZoneShape {
    Cuboid,
    Cylinder(f32),
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub struct Zone {
    name: Option<String>,
    #[serde(default)]
    kind: ZoneKind,
    min_height: i32,
    max_height: i32,
    points: Vec<ZonePoint>,
    shape: Option<ZoneShape>,
}

impl Component for Zone {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, ctx| {
            let zone = world
                .entity(ctx.entity)
                .get::<Zone>()
                .expect("Zone component should exist")
                .clone();

            let center = zone.center();
            let transform = Transform::from_translation(center);
            let collider = zone.collider();

            let zone_bundle = (
                transform,
                ZoneKindVariant::from(&zone.kind),
                collider,
                Sensor,
                CollisionEventsEnabled,
            );

            world.commands().entity(ctx.entity).insert(zone_bundle);
        })
    }
}
impl Zone {
    pub fn new(
        name: Option<String>,
        kind: ZoneKind,
        min_height: i32,
        max_height: i32,
        points: Vec<ZonePoint>,
        shape: Option<ZoneShape>,
    ) -> Self {
        Self {
            name,
            kind,
            min_height,
            max_height,
            points,
            shape,
        }
    }
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    pub fn name_component(&self) -> Name {
        Name::new(
            self.name
                .clone()
                .unwrap_or_else(|| format!("Unnamed-{:?}", self.kind)),
        )
    }
    pub fn kind(&self) -> &ZoneKind {
        &self.kind
    }
    pub fn kind_mut(&mut self) -> &mut ZoneKind {
        &mut self.kind
    }

    /// Generate vertices for the zone's points at min and max height
    fn vertices(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.points.iter().flat_map(|node| {
            [
                Vec3::new(node.x as f32, self.min_height as f32, node.y as f32),
                Vec3::new(node.x as f32, self.max_height as f32, node.y as f32),
            ]
        })
    }

    /// Calculate the center point of the zone for positioning
    pub fn center(&self) -> Vec3 {
        if let Some(shape) = &self.shape {
            match shape {
                ZoneShape::Cuboid => {
                    if self.points.len() != 2 {
                        panic!("Cuboid shape requires exactly 2 points");
                    }
                    let point1 = Vec3::new(
                        self.points[0].x as f32,
                        self.min_height as f32,
                        self.points[0].y as f32,
                    );
                    let point2 = Vec3::new(
                        self.points[1].x as f32,
                        self.max_height as f32,
                        self.points[1].y as f32,
                    );
                    (point2 - point1) / 2.0
                }
                ZoneShape::Cylinder(_) => {
                    if self.points.len() != 1 {
                        panic!("Cylinder shape requires exactly 1 node");
                    }
                    let height = (self.max_height - self.min_height).abs();
                    Vec3::new(
                        self.points[0].x as f32,
                        (self.min_height + height / 2) as f32,
                        self.points[0].y as f32,
                    )
                }
            }
        } else {
            let (sum, count) = self
                .vertices()
                .fold((Vec3::ZERO, 0), |(sum, count), v| (sum + v, count + 1));
            sum / count.max(1) as f32
        }
    }

    fn collider(&self) -> Collider {
        if let Some(shape) = &self.shape {
            match shape {
                ZoneShape::Cuboid => {
                    if self.points.len() != 2 {
                        panic!("Cuboid shape requires exactly 2 points");
                    }
                    let point1 = Vec3::new(
                        self.points[0].x as f32,
                        self.min_height as f32,
                        self.points[0].y as f32,
                    );
                    let point2 = Vec3::new(
                        self.points[1].x as f32,
                        self.max_height as f32,
                        self.points[1].y as f32,
                    );
                    let cuboid = Cuboid::from_corners(point1, point2);
                    Collider::from(cuboid)
                }
                ZoneShape::Cylinder(radius) => {
                    if self.points.len() != 1 {
                        panic!("Cylinder shape requires exactly 1 node");
                    }
                    let height = (self.max_height - self.min_height).abs();
                    let cylinder = Cylinder::new(*radius, height as f32);
                    Collider::from(cylinder)
                }
            }
        } else {
            let center = self.center();
            let centered_vertices: Vec<Vec3> = self.vertices().map(|v| v - center).collect();
            Collider::convex_hull(centered_vertices).unwrap()
        }
    }
}

#[derive(Asset, Clone, Debug, Default, Deref, Deserialize, Resource, TypePath)]
pub struct ZoneList(Vec<Zone>);

#[derive(Clone, Component, Default, Deref, Reflect)]
pub struct ZoneListHandle(Handle<ZoneList>);
impl From<Handle<ZoneList>> for ZoneListHandle {
    fn from(handle: Handle<ZoneList>) -> Self {
        Self(handle)
    }
}

#[derive(Clone, Default, Deref, DerefMut, Reflect, Resource)]
#[reflect(Resource)]
pub struct NamedZones(HashMap<String, Entity>);
