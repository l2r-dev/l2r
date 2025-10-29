use super::{GeoVec3, NavigationDirection};
use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};

/// Represents a 2D vector with integer coordinates, typically used for grid-based positioning (GeoData).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Reflect)]
pub struct GeoPoint {
    pub x: i32,
    pub y: i32,
}

impl GeoPoint {
    /// Creates a new instance of `GeoPoint`.
    pub fn new(x: i32, y: i32) -> Self {
        GeoPoint { x, y }
    }

    /// Returns a new `GeoPoint` with the absolute values of each component.
    pub fn abs(self) -> GeoPoint {
        GeoPoint {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    /// Calculates the absolute difference between this vector and another.
    pub fn delta(&self, other: &GeoPoint) -> GeoPoint {
        (*other - *self).abs()
    }

    /// Calculates the Euclidean distance to another point, rounded to the nearest integer.
    pub fn distance(&self, other: &GeoPoint) -> i32 {
        let d = self.delta(other);
        ((d.x * d.x + d.y * d.y) as f32).sqrt() as i32
    }

    /// Calculates the Manhattan distance to another point.
    pub fn manhattan_distance(&self, other: &GeoPoint) -> i32 {
        let d = self.delta(other);
        // cause diagonal distance is 1.4 times bigger than straight
        // by pythagorean theorem: sqrt(1^2 + 1^2) = sqrt(2) = 1.4
        // we count in i32 so we can't use sqrt, so we just multiply by 10
        (d.x + d.y) * 10
    }

    /// Returns a vector pointing in the direction from this point to another.
    ///
    /// Each component will be either 1 or -1, representing the direction of movement along that axis.
    pub fn step(&self, other: &GeoPoint) -> GeoPoint {
        GeoPoint {
            x: if self.x < other.x { 1 } else { -1 },
            y: if self.y < other.y { 1 } else { -1 },
        }
    }

    /// Returns the maximum absolute value among the vector's components.
    pub fn max_component(&self) -> i32 {
        self.x.abs().max(self.y.abs())
    }

    /// Determines the direction from this point to another.
    pub fn direction(self, other: &GeoPoint) -> NavigationDirection {
        NavigationDirection::from_offset(other.x - self.x, other.y - self.y)
    }

    /// Creates an iterator that yields points along a line from this point to the end point.
    ///
    /// Uses Bresenham's line algorithm to generate the points.
    pub fn line_to(&self, end: &GeoPoint) -> LinePointIterator {
        LinePointIterator::new(*self, *end)
    }
}

impl Add for GeoPoint {
    type Output = GeoPoint;
    fn add(self, other: GeoPoint) -> Self::Output {
        GeoPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for GeoPoint {
    type Output = GeoPoint;
    fn sub(self, other: GeoPoint) -> Self::Output {
        GeoPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<i32> for GeoPoint {
    type Output = GeoPoint;
    fn mul(self, other: i32) -> Self::Output {
        GeoPoint {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

/// Iterator that yields points along a 2D line using Bresenham's algorithm.
pub struct LinePointIterator {
    current_point: GeoPoint,
    end_point: GeoPoint,
    delta: GeoPoint,
    step: GeoPoint,
    error: i32,
    is_first_point: bool,
}

impl LinePointIterator {
    fn new(start: GeoPoint, end: GeoPoint) -> Self {
        let delta = start.delta(&end);
        let step = start.step(&end);
        let initial_error = delta.max_component() / 2;

        Self {
            current_point: start,
            end_point: end,
            delta,
            step,
            error: initial_error,
            is_first_point: true,
        }
    }
}

impl Iterator for LinePointIterator {
    type Item = GeoPoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_first_point {
            self.is_first_point = false;
            return Some(self.current_point);
        }

        if self.current_point == self.end_point {
            return None;
        }

        let max_delta = self.delta.max_component();

        if max_delta == self.delta.x {
            self.current_point.x += self.step.x;
            self.error += self.delta.y;
            if self.error >= self.delta.x {
                self.current_point.y += self.step.y;
                self.error -= self.delta.x;
            }
        } else {
            self.current_point.y += self.step.y;
            self.error += self.delta.x;
            if self.error >= self.delta.y {
                self.current_point.x += self.step.x;
                self.error -= self.delta.y;
            }
        }

        Some(self.current_point)
    }
}

impl From<GeoVec3> for GeoPoint {
    fn from(value: GeoVec3) -> Self {
        GeoPoint {
            x: value.point.x,
            y: value.point.y,
        }
    }
}
