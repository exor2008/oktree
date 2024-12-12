//! Bounding primitives.
//!
//! [`TUVec3`], [`BVec3`], [`Aabb`]

use crate::{Position, TreeError};
use alloc::fmt::{self, Debug, Display};
use core::ops::{Add, AddAssign, BitAnd, Div, DivAssign, Mul, MulAssign, Shr, Sub, SubAssign};
use num::{cast, Integer, NumCast, Saturating, Unsigned as NumUnsigned};

pub trait Unsigned:
    Integer
    + NumUnsigned
    + NumCast
    + Saturating
    + Add
    + AddAssign
    + Sub
    + SubAssign
    + Div
    + DivAssign
    + Mul
    + MulAssign
    + Shr<Self, Output = Self>
    + BitAnd<Self, Output = Self>
    + Copy
    + Display
    + Debug
    + Default
{
}
impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}
impl Unsigned for usize {}

/// Tree Unsigned Vec3
///
/// Inner typy shuld be any [`Unsigned`](num::Unsigned):
/// `u8`, `u16`, `u32`, `u64`, `u128`, `usize`.
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3<U: Unsigned> {
    pub x: U,
    pub y: U,
    pub z: U,
}

impl<U: Unsigned> Add for TUVec3<U> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        TUVec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<U: Unsigned> Sub for TUVec3<U> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        TUVec3 {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
            z: self.z.saturating_sub(other.z),
        }
    }
}

impl<U: Unsigned> AddAssign for TUVec3<U> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl<U: Unsigned> SubAssign for TUVec3<U> {
    fn sub_assign(&mut self, other: Self) {
        self.x = self.x.saturating_sub(other.x);
        self.y = self.y.saturating_sub(other.y);
        self.z = self.z.saturating_sub(other.z);
    }
}

impl<U: Unsigned> Mul<U> for TUVec3<U> {
    type Output = Self;

    fn mul(self, other: U) -> Self {
        TUVec3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl<U: Unsigned> MulAssign<U> for TUVec3<U> {
    fn mul_assign(&mut self, other: U) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
    }
}

impl<U: Unsigned> Div<U> for TUVec3<U> {
    type Output = Self;

    fn div(self, other: U) -> Self {
        TUVec3 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl<U: Unsigned> DivAssign<U> for TUVec3<U> {
    fn div_assign(&mut self, other: U) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
    }
}

impl<U: Unsigned> Display for TUVec3<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uvec3: x: {}, y: {}, z: {}", self.x, self.y, self.z)
    }
}

impl<U: Unsigned> TUVec3<U> {
    pub fn new(x: U, y: U, z: U) -> Self {
        TUVec3 { x, y, z }
    }

    pub fn splat(size: U) -> Self {
        TUVec3 {
            x: size,
            y: size,
            z: size,
        }
    }

    pub fn zero() -> Self {
        TUVec3 {
            x: cast(0).unwrap(),
            y: cast(0).unwrap(),
            z: cast(0).unwrap(),
        }
    }

    pub fn lt(&self, other: &Self) -> BVec3 {
        BVec3::new(self.x < other.x, self.y < other.y, self.z < other.z)
    }

    pub fn gt(&self, other: &Self) -> BVec3 {
        BVec3::new(self.x > other.x, self.y > other.y, self.z > other.z)
    }

    pub fn le(&self, other: &Self) -> BVec3 {
        BVec3::new(self.x <= other.x, self.y <= other.y, self.z <= other.z)
    }

    pub fn ge(&self, other: &Self) -> BVec3 {
        BVec3::new(self.x >= other.x, self.y >= other.y, self.z >= other.z)
    }

    #[inline]
    /// Checks if [`Aabb`] creted from this [`TUVec3`] and `half_size` will have all dimensions positive.
    pub fn is_positive_aabb(&self, half_size: U) -> bool {
        self.x >= half_size && self.y >= half_size && self.z >= half_size
    }

    /// Creates [`Aabb`] with size of 1 from current [`TUVec3`].
    pub fn unit_aabb(&self) -> Aabb<U> {
        let max = TUVec3::new(
            self.x + cast(1).unwrap(),
            self.y + cast(1).unwrap(),
            self.z + cast(1).unwrap(),
        );
        Aabb { min: *self, max }
    }
}

/// Boolean Vec3 mask.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BVec3 {
    x: bool,
    y: bool,
    z: bool,
}

impl BVec3 {
    fn new(x: bool, y: bool, z: bool) -> Self {
        BVec3 { x, y, z }
    }

    pub fn all(&self) -> bool {
        self.x && self.y && self.z
    }

    pub fn any(&self) -> bool {
        self.x || self.y || self.z
    }

    pub fn none(&self) -> bool {
        !self.x && !self.y && !self.z
    }
}

/// Axis Aligned Bounding Box
///
/// Resulting Aabb should be positive and it's dimensions should be the power of 2.
/// Inner type shuld be any [`Unsigned`](num::Unsigned):
/// `u8`, `u16`, `u32`, `u64`, `u128`, `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aabb<U: Unsigned> {
    pub min: TUVec3<U>,
    pub max: TUVec3<U>,
}

impl<U: Unsigned> Default for Aabb<U> {
    fn default() -> Self {
        Self {
            min: TUVec3::new(cast(0).unwrap(), cast(0).unwrap(), cast(0).unwrap()),
            max: TUVec3::new(cast(1).unwrap(), cast(1).unwrap(), cast(1).unwrap()),
        }
    }
}

impl<U: Unsigned> Display for Aabb<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Aabb(min: {}, max: {})", self.min, self.max)
    }
}

impl<U: Unsigned> Aabb<U> {
    /// Creates a new [Aabb] object without any checks
    pub fn new_unchecked(center: TUVec3<U>, half_size: U) -> Self {
        Aabb {
            min: TUVec3::new(
                center.x.saturating_sub(half_size),
                center.y.saturating_sub(half_size),
                center.z.saturating_sub(half_size),
            ),
            max: TUVec3::new(
                center.x + half_size,
                center.y + half_size,
                center.z + half_size,
            ),
        }
    }

    /// Creates a new [`Aabb`] object
    ///
    /// Checks that it's dimensions are positive
    /// and are powers of 2.
    pub fn new(center: TUVec3<U>, half_size: U) -> Result<Self, TreeError> {
        if !center.is_positive_aabb(half_size) {
            Err(TreeError::NotPositive(
                "Center: {center}, half size: {half_size}".into(),
            ))
        } else if !is_power2(half_size) {
            Err(TreeError::NotPower2("half size: {half_size}".into()))
        } else {
            Ok(Self::new_unchecked(center, half_size))
        }
    }

    /// Creates a new [`Aabb`] object from a min and max
    pub fn from_min_max(min: TUVec3<U>, max: TUVec3<U>) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> TUVec3<U> {
        TUVec3::new(
            (self.min.x + self.max.x) >> cast(1).unwrap(),
            (self.min.y + self.max.y) >> cast(1).unwrap(),
            (self.min.z + self.max.z) >> cast(1).unwrap(),
        )
    }

    #[inline]
    pub fn split(&self) -> [Aabb<U>; 8] {
        let center = self.center();
        [
            Aabb::from_min_max(
                TUVec3::new(self.min.x, self.min.y, self.min.z),
                TUVec3::new(center.x, center.y, center.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(center.x, self.min.y, self.min.z),
                TUVec3::new(self.max.x, center.y, center.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(self.min.x, center.y, self.min.z),
                TUVec3::new(center.x, self.max.y, center.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(center.x, center.y, self.min.z),
                TUVec3::new(self.max.x, self.max.y, center.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(self.min.x, self.min.y, center.z),
                TUVec3::new(center.x, center.y, self.max.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(center.x, self.min.y, center.z),
                TUVec3::new(self.max.x, center.y, self.max.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(self.min.x, center.y, center.z),
                TUVec3::new(center.x, self.max.y, self.max.z),
            ),
            Aabb::from_min_max(
                TUVec3::new(center.x, center.y, center.z),
                TUVec3::new(self.max.x, self.max.y, self.max.z),
            ),
        ]
    }

    fn _split(&self, i: usize, center: TUVec3<U>) -> Aabb<U> {
        let x_mask = (i & 0b1) != 0;
        let y_mask = (i & 0b10) != 0;
        let z_mask = (i & 0b100) != 0;

        Aabb {
            min: TUVec3::new(
                if x_mask { center.x } else { self.min.x },
                if y_mask { center.y } else { self.min.y },
                if z_mask { center.z } else { self.min.z },
            ),
            max: TUVec3::new(
                if x_mask { self.max.x } else { center.x },
                if y_mask { self.max.y } else { center.y },
                if z_mask { self.max.z } else { center.z },
            ),
        }
    }

    /// Checks if the aabb contains a [`position`](TUVec3).
    pub fn contains(&self, position: &TUVec3<U>) -> bool {
        let lemin = self.min.le(position);
        let gtmax = self.max.gt(position);

        lemin.all() && gtmax.all()
    }

    /// Checks if this volume overlaps with another [`Aabb`].
    pub fn overlaps(&self, other: &Aabb<U>) -> bool {
        self.max.x.min(other.max.x) > self.min.x.max(other.min.x)
            && self.max.y.min(other.max.y) > self.min.y.max(other.min.y)
            && self.max.z.min(other.max.z) > self.min.z.max(other.min.z)
    }

    pub fn unit(&self) -> bool {
        self.max.x - self.min.x == cast(1).unwrap()
    }

    pub fn size(&self) -> U {
        self.max.x - self.min.x
    }
}

/// Check if `half_size` is the power of 2.
///
/// Used in [`aabb creation`](Aabb::new) checks.
pub fn is_power2<U: Unsigned>(mut half_size: U) -> bool {
    if half_size < cast(2).unwrap() {
        return false;
    }

    while half_size > cast(1).unwrap() {
        if half_size & cast(0b1).unwrap() != cast(0).unwrap() {
            return false;
        }

        half_size = half_size >> cast(1).unwrap();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{is_power2, Aabb, TUVec3};

    #[test]
    fn test_aabb_contains() {
        let aabb = Aabb::new_unchecked(TUVec3::new(8, 8, 8), 8u16);
        assert!(aabb.contains(&TUVec3::zero()));

        assert!(aabb.contains(&TUVec3::new(8, 8, 8)));

        assert!(!aabb.contains(&TUVec3::new(16, 16, 16)));

        assert!(!aabb.contains(&TUVec3::new(0, 16, 8)));
    }

    #[test]
    fn test_ispower2() {
        assert!(!is_power2(0u32));
        assert!(!is_power2(1u32));
        assert!(is_power2(2u8));
        assert!(!is_power2(3u32));
        assert!(is_power2(4u16));
        assert!(!is_power2(5u16));
        assert!(is_power2(8u8));
        assert!(!is_power2(1023usize));
        assert!(is_power2(1024usize));
        assert!(!is_power2(1025usize));
    }

    #[test]
    fn test_aabb_constructor() {
        // Ok
        assert!(Aabb::new(TUVec3::splat(2u8), 2).is_ok());

        // Negative dimensions
        assert!(Aabb::new(TUVec3::splat(16u16), 64).is_err());

        // 7 is not the power of 2
        assert!(Aabb::new(TUVec3::splat(16u16), 7).is_err());
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3u8(pub TUVec3<u8>);
impl TUVec3u8 {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        TUVec3u8(TUVec3::new(x, y, z))
    }
}
impl Position for TUVec3u8 {
    type U = u8;
    fn position(&self) -> TUVec3<u8> {
        self.0
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3u16(pub TUVec3<u16>);
impl TUVec3u16 {
    pub fn new(x: u16, y: u16, z: u16) -> Self {
        TUVec3u16(TUVec3::new(x, y, z))
    }
}
impl Position for TUVec3u16 {
    type U = u16;
    fn position(&self) -> TUVec3<u16> {
        self.0
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3u32(pub TUVec3<u32>);
impl TUVec3u32 {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        TUVec3u32(TUVec3::new(x, y, z))
    }
}
impl Position for TUVec3u32 {
    type U = u32;
    fn position(&self) -> TUVec3<u32> {
        self.0
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3u64(pub TUVec3<u64>);
impl TUVec3u64 {
    pub fn new(x: u64, y: u64, z: u64) -> Self {
        TUVec3u64(TUVec3::new(x, y, z))
    }
}
impl Position for TUVec3u64 {
    type U = u64;
    fn position(&self) -> TUVec3<u64> {
        self.0
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TUVec3u128(pub TUVec3<u128>);
impl TUVec3u128 {
    pub fn new(x: u128, y: u128, z: u128) -> Self {
        TUVec3u128(TUVec3::new(x, y, z))
    }
}
impl Position for TUVec3u128 {
    type U = u128;
    fn position(&self) -> TUVec3<u128> {
        self.0
    }
}
