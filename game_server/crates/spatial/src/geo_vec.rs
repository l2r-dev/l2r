use super::GeoPoint;
use crate::NavigationDirection;
use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};

/// Represents a 3D vector with integer coordinates, used for grid-based positioning (GeoData)
/// Bevy's [`Vec3`] <> [`GameVec3`] <> [`GeoVec3`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Reflect)]
pub struct GeoVec3 {
    pub point: GeoPoint,
    pub height: i32,
}

impl Add<GeoVec3> for GeoVec3 {
    type Output = GeoVec3;
    fn add(self, other: GeoVec3) -> Self::Output {
        GeoVec3 {
            point: self.point + other.point,
            height: self.height + other.height,
        }
    }
}

impl Sub for GeoVec3 {
    type Output = GeoVec3;
    fn sub(self, other: GeoVec3) -> Self::Output {
        GeoVec3 {
            point: self.point - other.point,
            height: self.height - other.height,
        }
    }
}

impl Mul<i32> for GeoVec3 {
    type Output = GeoVec3;
    fn mul(self, other: i32) -> Self::Output {
        GeoVec3 {
            point: self.point * other,
            height: self.height * other,
        }
    }
}

impl GeoVec3 {
    pub fn new(point: GeoPoint, height: i32) -> Self {
        GeoVec3 { point, height }
    }

    pub fn point(&self) -> GeoPoint {
        self.point
    }

    pub fn abs(self) -> GeoVec3 {
        GeoVec3 {
            point: self.point.abs(),
            height: self.height.abs(),
        }
    }

    /// Calculates the absolute difference between this vector and another.
    pub fn delta(&self, other: &GeoVec3) -> GeoVec3 {
        (*other - *self).abs()
    }

    /// Calculates the Euclidean distance to another point, rounded to the nearest integer.
    pub fn distance(&self, other: &GeoVec3) -> i32 {
        let d = self.delta(other);
        ((d.point.x * d.point.x + d.point.y * d.point.y + d.height * d.height) as f32).sqrt() as i32
    }

    /// Calculates the squared Euclidean distance to another point.
    /// This is more efficient than `distance()` when you only need to compare distances,
    /// as it avoids the expensive square root calculation.
    pub fn distance_squared(&self, other: &GeoVec3) -> i32 {
        let d = self.delta(other);
        d.point.x * d.point.x + d.point.y * d.point.y + d.height * d.height
    }

    /// Calculates the Manhattan distance to another point.
    pub fn manhattan_distance(&self, other: &GeoVec3) -> i32 {
        let d = self.delta(other);
        // cause diagonal distance is 1.4 times bigger than straight
        // by pythagorean theorem: sqrt(1^2 + 1^2) = sqrt(2) = 1.4
        // we count in i32 so we can't use sqrt, so we just multiply by 10
        (d.point.x + d.point.y + d.height) * 10
    }

    /// Calculates the Chebyshev distance to another point.
    pub fn chebyshev_distance(&self, other: &GeoVec3) -> i32 {
        let d = self.delta(other);

        // log::error!("chebyshev_distance: {:?}", distance);
        d.point.x.max(d.point.y).max(d.height)
    }

    /// Returns a vector indicating the direction to move from this point to another.
    ///
    /// Each component will be either 1 or -1, representing the direction of movement
    /// along that axis.
    pub fn step(&self, other: &GeoVec3) -> GeoVec3 {
        GeoVec3 {
            point: self.point.step(&other.point),
            height: if self.height < other.height { 1 } else { -1 },
        }
    }

    /// Returns the maximum absolute value among the vector's components.
    pub fn max_component(&self) -> i32 {
        self.point.max_component().max(self.height.abs())
    }

    /// Gets the adjacent position in the specified direction.
    pub fn adjacent_position_in(&self, direction: NavigationDirection, step: Option<i32>) -> Self {
        let (dx, dy) = direction.offset(step.unwrap_or(1));
        GeoVec3::new(
            GeoPoint::new(self.point.x + dx, self.point.y + dy),
            self.height,
        )
    }

    /// Calculates the next point towards a target, moving by a specified step size.
    fn next_towards(&self, other: GeoVec3, step_size: i32) -> GeoVec3 {
        let (dx, dy) = self.direction(other).offset(step_size);
        GeoVec3::new(
            GeoPoint::new(self.point.x + dx, self.point.y + dy),
            self.height,
        )
    }

    /// Gets the next step towards a target, moving by 1 unit.
    pub fn next_step_towards(&self, other: GeoVec3) -> GeoVec3 {
        self.next_towards(other, 1)
    }

    /// Determines the direction from this point to another.
    pub fn direction(self, other: GeoVec3) -> NavigationDirection {
        NavigationDirection::from_offset(other.point.x - self.point.x, other.point.y - self.point.y)
    }

    /// Creates an iterator that yields points along a line from this point to the end point.
    ///
    /// This uses Bresenham's line algorithm to generate the points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spatial::{GeoVec3, GeoPoint};
    /// let start = GeoVec3::new(GeoPoint::new(0, 0), 0);
    /// let end = GeoVec3::new(GeoPoint::new(2, 2), 0);
    /// let line: Vec<GeoVec3> = start.line_to(&end).collect();
    /// assert_eq!(line, vec![
    ///     GeoVec3::new(GeoPoint::new(0, 0), 0),
    ///     GeoVec3::new(GeoPoint::new(1, 1), 0),
    ///     GeoVec3::new(GeoPoint::new(2, 2), 0),
    /// ]);
    /// ```
    pub fn line_to(&self, end: GeoVec3) -> LineVecIterator {
        LineVecIterator::new(*self, end)
    }
}

/// An iterator that yields points along a 3D line using Bresenham's algorithm.
pub struct LineVecIterator {
    current_point: GeoVec3,
    end_point: GeoVec3,
    delta: GeoVec3,
    step: GeoVec3,
    error_xy: i32,
    error_xz: i32,
    is_first_point: bool,
}

impl LineVecIterator {
    fn new(start: GeoVec3, end: GeoVec3) -> Self {
        let delta = start.delta(&end);
        let step = start.step(&end);
        let max_delta = delta.max_component();
        let initial_error = max_delta / 2;

        Self {
            current_point: start,
            end_point: end,
            delta,
            step,
            error_xy: initial_error,
            error_xz: initial_error,
            is_first_point: true,
        }
    }
}

/// Bresenham's line algorithm
impl Iterator for LineVecIterator {
    type Item = GeoVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_first_point {
            self.is_first_point = false;
            return Some(self.current_point);
        }

        if self.current_point == self.end_point {
            return None;
        }

        let max_delta = self.delta.max_component();

        if max_delta == self.delta.point.x {
            self.current_point.point.x += self.step.point.x;
            self.error_xy += self.delta.point.y;
            self.error_xz += self.delta.height;
            if self.error_xy >= self.delta.point.x {
                self.current_point.point.y += self.step.point.y;
                self.error_xy -= self.delta.point.x;
            }
            if self.error_xz >= self.delta.point.x {
                self.current_point.height += self.step.height;
                self.error_xz -= self.delta.point.x;
            }
        } else if max_delta == self.delta.point.y {
            self.current_point.point.y += self.step.point.y;
            self.error_xy += self.delta.point.x;
            self.error_xz += self.delta.height;
            if self.error_xy >= self.delta.point.y {
                self.current_point.point.x += self.step.point.x;
                self.error_xy -= self.delta.point.y;
            }
            if self.error_xz >= self.delta.point.y {
                self.current_point.height += self.step.height;
                self.error_xz -= self.delta.point.y;
            }
        } else {
            self.current_point.height += self.step.height;
            self.error_xy += self.delta.point.x;
            self.error_xz += self.delta.point.y;
            if self.error_xy >= self.delta.height {
                self.current_point.point.x += self.step.point.x;
                self.error_xy -= self.delta.height;
            }
            if self.error_xz >= self.delta.height {
                self.current_point.point.y += self.step.point.y;
                self.error_xz -= self.delta.height;
            }
        }

        Some(self.current_point)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geovec3_new() {
        let vec = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        assert_eq!(vec.point.x, 42722);
        assert_eq!(vec.point.y, 37551);
        assert_eq!(vec.height, -4233);
    }

    #[test]
    fn test_geovec3_sub() {
        let vec1 = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(42720, 37550), -4230);
        let result = vec1 - vec2;
        assert_eq!(result, GeoVec3::new(GeoPoint::new(2, 1), -3));
    }

    #[test]
    fn test_geovec3_mul() {
        let vec = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let result = vec * 2;
        assert_eq!(result, GeoVec3::new(GeoPoint::new(85444, 75102), -8466));
    }

    #[test]
    fn test_geovec3_abs() {
        let vec = GeoVec3::new(GeoPoint::new(-42722, -37551), -4233);
        let result = vec.abs();
        assert_eq!(result, GeoVec3::new(GeoPoint::new(42722, 37551), 4233));
    }

    #[test]
    fn test_geovec3_delta() {
        let vec1 = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(42725, 37554), -4230);
        let result = vec1.delta(&vec2);
        assert_eq!(result, GeoVec3::new(GeoPoint::new(3, 3), 3));
    }

    #[test]
    fn test_geovec3_step() {
        let vec1 = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(42725, 37550), -4233);
        let result = vec1.step(&vec2);
        assert_eq!(result, GeoVec3::new(GeoPoint::new(1, -1), -1));
    }

    #[test]
    fn test_geovec3_max_component() {
        let vec = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        assert_eq!(vec.max_component(), 42722);
    }

    #[test]
    fn test_geovec3_distance() {
        let vec1 = GeoVec3::new(GeoPoint::new(10, 5), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(20, 10), -4233);
        assert_eq!(vec1.distance(&vec2), 11);
    }

    #[test]
    fn test_geovec3_get_next_towards() {
        let vec1 = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(42732, 37561), -4233);
        let result = vec1.next_towards(vec2, 1);
        assert_eq!(result, GeoVec3::new(GeoPoint::new(42723, 37552), -4233));
    }

    #[test]
    fn test_line_iterator() {
        let start = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let end = GeoVec3::new(GeoPoint::new(42724, 37553), -4233);
        let line: Vec<GeoVec3> = start.line_to(&end).collect();
        assert_eq!(
            line,
            vec![
                GeoVec3::new(GeoPoint::new(42722, 37551), -4233),
                GeoVec3::new(GeoPoint::new(42723, 37552), -4233),
                GeoVec3::new(GeoPoint::new(42724, 37553), -4233),
            ]
        );
    }

    #[test]
    fn test_line_iterator_diagonal() {
        let start = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let end = GeoVec3::new(GeoPoint::new(42725, 37552), -4233);
        let line: Vec<GeoVec3> = start.line_to(&end).collect();
        assert_eq!(
            line,
            vec![
                GeoVec3::new(GeoPoint::new(42722, 37551), -4233),
                GeoVec3::new(GeoPoint::new(42723, 37551), -4233),
                GeoVec3::new(GeoPoint::new(42724, 37552), -4233),
                GeoVec3::new(GeoPoint::new(42725, 37552), -4233),
            ]
        );
    }

    #[test]
    fn test_line_iterator_3d() {
        let start = GeoVec3::new(GeoPoint::new(42722, 37551), -4233);
        let end = GeoVec3::new(GeoPoint::new(42724, 37553), -4231);
        let line: Vec<GeoVec3> = start.line_to(&end).collect();
        assert_eq!(
            line,
            vec![
                GeoVec3::new(GeoPoint::new(42722, 37551), -4233),
                GeoVec3::new(GeoPoint::new(42723, 37552), -4232),
                GeoVec3::new(GeoPoint::new(42724, 37553), -4231),
            ]
        );
    }

    #[test]
    fn test_get_adjacent_position_in_default_step() {
        let position = GeoVec3::new(GeoPoint::new(20000, 17000), -4232);
        let direction = NavigationDirection::NORTH;
        let new_position = position.adjacent_position_in(direction, None);
        assert_eq!(
            new_position,
            GeoVec3::new(GeoPoint::new(20000, 16999), -4232)
        );
    }

    #[test]
    fn test_get_adjacent_position_in_diagonal_default_step() {
        let position = GeoVec3::new(GeoPoint::new(20000, 17000), -4232);
        let direction = NavigationDirection::NORTH_EAST;
        let new_position = position.adjacent_position_in(direction, None);
        assert_eq!(
            new_position,
            GeoVec3::new(GeoPoint::new(20001, 16999), -4232)
        );
    }

    #[test]
    fn test_get_adjacent_position_in_east_default_step() {
        let position = GeoVec3::new(GeoPoint::new(20000, 17000), -4232);
        let direction = NavigationDirection::EAST;
        let new_position = position.adjacent_position_in(direction, None);
        assert_eq!(
            new_position,
            GeoVec3::new(GeoPoint::new(20001, 17000), -4232)
        );
    }

    // manhattan distance
    #[test]
    fn test_manhattan_distance() {
        let vec1 = GeoVec3::new(GeoPoint::new(0, 0), -4233);
        let vec2 = GeoVec3::new(GeoPoint::new(1, 0), -4233);
        assert_eq!(vec1.manhattan_distance(&vec2), 10);
    }
}
