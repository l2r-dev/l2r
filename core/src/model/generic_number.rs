use num_traits::{Bounded, NumCast, One, Unsigned, Zero};
use std::{
    fmt,
    hash::{BuildHasher, Hash, Hasher},
    ops::{Add, AddAssign, Deref, Div, Mul, Sub, SubAssign},
    str::FromStr,
};

pub trait GenericNumber<V>:
    Clone
    + Copy
    + fmt::Debug
    + Default
    + Eq
    + Hash
    + PartialEq
    + PartialOrd
    + Ord
    + FromStr
    + From<usize>
    + From<i32>
    + From<V>
    + Into<V>
    + fmt::Display
    + Deref
    + Into<sea_orm::Value>
    + sea_orm::TryGetable
    + sea_orm::sea_query::ValueType
    + Add
    + Sub
    + Mul
    + Div
    + AddAssign
    + SubAssign
where
    V: Unsigned + One + Zero + Copy + fmt::Debug + NumCast + FromStr + Bounded,
{
    /// Retrieves the underlying numeric value.
    fn value(&self) -> V;

    /// Parses a filename to extract an ID range.
    fn range_from_filename(s: &str) -> GenericNumRange<Self, V>
    where
        Self: Sized,
    {
        let s = s.trim_end_matches(".json");
        let mut parts = s.split('-');
        let start: u32 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or_default();
        let end: u32 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or_default();
        let start_id = Self::from(V::from(start).unwrap_or_else(|| V::zero()));
        let end_id = Self::from(V::from(end).unwrap_or_else(|| V::zero()));
        GenericNumRange::new(start_id, end_id)
    }

    fn range(&self) -> GenericNumRange<Self, V>
    where
        Self: Sized,
    {
        let range_size = V::from(100).unwrap_or_else(|| V::one());
        GenericNumRange::from_id(*self, range_size)
    }

    fn placeholder() -> Self {
        Self::from(V::max_value())
    }
}

#[derive(Clone)]
pub struct GenericNumRange<I: GenericNumber<V>, V>
where
    V: Unsigned + One + Zero + Copy + fmt::Debug + NumCast + FromStr + Bounded,
{
    start: I,
    end: I,
    exhausted: bool,
    phantom: std::marker::PhantomData<V>,
}

impl<I: GenericNumber<V>, V> GenericNumRange<I, V>
where
    V: Unsigned + One + Zero + Copy + fmt::Debug + NumCast + FromStr + Bounded,
{
    pub fn new(start: I, end: I) -> Self {
        Self {
            start,
            end,
            exhausted: false,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn from_id(id: I, range_size: V) -> Self {
        let id_value = id.value();
        let start_value = (id_value / range_size) * range_size;
        let end_value = start_value + range_size - V::one();

        let start_id = I::from(start_value);
        let end_id = I::from(end_value);

        Self::new(start_id, end_id)
    }
}

impl<I: GenericNumber<V> + PartialEq, V> Iterator for GenericNumRange<I, V>
where
    V: Unsigned + One + Zero + Copy + fmt::Debug + NumCast + FromStr + Bounded,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        let current = self.start.value();
        let end = self.end.value();

        if current == end {
            self.exhausted = true;
        } else {
            self.start = (current + V::one()).into();
        }

        Some(current.into())
    }
}

impl<I: GenericNumber<V> + fmt::Display, V> fmt::Display for GenericNumRange<I, V>
where
    V: Unsigned + One + Zero + Copy + fmt::Debug + NumCast + FromStr + fmt::Display + Bounded,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}-{:05}", self.start.value(), self.end.value())
    }
}

#[macro_export]
macro_rules! impl_std_math_operations {
    ($id:ident, $v:ident) => {
        impl Add for $id {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self::from(self.value() + rhs.value())
            }
        }

        impl Sub for $id {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self::from(self.value() - rhs.value())
            }
        }

        impl Mul for $id {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self::from(self.value() * rhs.value())
            }
        }

        impl Div for $id {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self::from(self.value() / rhs.value())
            }
        }

        impl AddAssign for $id {
            fn add_assign(&mut self, rhs: Self) {
                *self = Self::from(self.value() + rhs.value());
            }
        }

        impl SubAssign for $id {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self::from(self.value() - rhs.value());
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_conversions {
    ($id:ident, $v:ident) => {
        impl From<u16> for $id {
            fn from(value: u16) -> Self {
                $id(value as $v)
            }
        }

        impl From<u32> for $id {
            fn from(value: u32) -> Self {
                $id(value as $v)
            }
        }

        impl From<i32> for $id {
            fn from(value: i32) -> Self {
                $id(value as $v)
            }
        }

        impl From<i64> for $id {
            fn from(value: i64) -> Self {
                $id(value as $v)
            }
        }

        impl From<usize> for $id {
            fn from(value: usize) -> Self {
                $id(value as $v)
            }
        }

        impl From<u64> for $id {
            fn from(value: u64) -> Self {
                $id(value as $v)
            }
        }

        impl FromStr for $id {
            type Err = std::num::ParseIntError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse::<$v>().map(Self)
            }
        }

        impl From<$id> for u16 {
            fn from(value: $id) -> Self {
                value.value() as u16
            }
        }

        impl From<$id> for u32 {
            fn from(value: $id) -> Self {
                value.value() as u32
            }
        }

        impl From<$id> for i32 {
            fn from(value: $id) -> Self {
                value.value() as i32
            }
        }

        impl From<$id> for i64 {
            fn from(value: $id) -> Self {
                value.value() as i64
            }
        }

        impl From<$id> for f32 {
            fn from(value: $id) -> Self {
                value.value() as f32
            }
        }

        impl From<f32> for $id {
            fn from(value: f32) -> Self {
                $id(value as $v)
            }
        }

        impl From<$id> for f64 {
            fn from(value: $id) -> Self {
                value.value() as f64
            }
        }

        impl From<f64> for $id {
            fn from(value: f64) -> Self {
                $id(value as $v)
            }
        }

        impl From<$id> for usize {
            fn from(value: $id) -> Self {
                value.value() as usize
            }
        }
    };
    ($id:ident, $v:ident, $t:ty) => {
        impl From<$t> for $id {
            fn from(value: $t) -> Self {
                $id(value as $v)
            }
        }
    };
}

#[derive(Clone, Default)]
pub struct SimpleNumberHasher(u64);

impl Hasher for SimpleNumberHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(byte as u64);
        }
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }
}

impl BuildHasher for SimpleNumberHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}
