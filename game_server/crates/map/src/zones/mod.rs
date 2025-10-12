use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
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
            .register_type::<ZoneKindVariant>();

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

#[derive(Clone, Component, Debug, Deserialize, Reflect, Serialize)]
pub struct Zone {
    name: Option<String>,
    #[serde(default)]
    kind: ZoneKind,
    min_height: i32,
    max_height: i32,
    points: Vec<ZonePoint>,
    shape: Option<ZoneShape>,
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
    pub fn build(
        &self,
    ) -> (
        Transform,
        ZoneKindVariant,
        Collider,
        Sensor,
        CollisionEventsEnabled,
    ) {
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
                    let collider = Collider::from(cuboid);
                    let middle_point = (point2 - point1) / 2.0;
                    let transform = Transform::from_translation(middle_point);
                    (
                        transform,
                        ZoneKindVariant::from(&self.kind),
                        collider,
                        Sensor,
                        CollisionEventsEnabled,
                    )
                }
                ZoneShape::Cylinder(radius) => {
                    if self.points.len() != 1 {
                        panic!("Cylinder shape requires exactly 1 node");
                    }
                    let height = (self.max_height - self.min_height).abs();
                    let cylinder = Cylinder::new(*radius, height as f32);
                    let collider = Collider::from(cylinder);
                    let transform = Transform::from_translation(Vec3::new(
                        self.points[0].x as f32,
                        (self.min_height + height / 2) as f32,
                        self.points[0].y as f32,
                    ));
                    (
                        transform,
                        ZoneKindVariant::from(&self.kind),
                        collider,
                        Sensor,
                        CollisionEventsEnabled,
                    )
                }
            }
        } else {
            let mut vertices = Vec::with_capacity(self.points.len() * 2);

            for node in &self.points {
                vertices.push(Vec3::new(
                    node.x as f32,
                    self.min_height as f32,
                    node.y as f32,
                ));
                vertices.push(Vec3::new(
                    node.x as f32,
                    self.max_height as f32,
                    node.y as f32,
                ));
            }

            // Calculate the center of the vertices
            let center = if !vertices.is_empty() {
                vertices.iter().sum::<Vec3>() / vertices.len() as f32
            } else {
                Vec3::ZERO
            };

            // Translate all vertices to be centered around the origin
            let centered_vertices: Vec<Vec3> = vertices.iter().map(|v| *v - center).collect();
            let collider = Collider::convex_hull(centered_vertices).unwrap();
            let transform = Transform::from_translation(center);
            (
                transform,
                ZoneKindVariant::from(&self.kind),
                collider,
                Sensor,
                CollisionEventsEnabled,
            )
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

#[derive(Clone, Default, Deref, DerefMut, Resource)]
pub struct NamedZones(HashMap<String, Entity>);
