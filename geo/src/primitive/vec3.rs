use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[cfg(test)]
use proptest::prelude::*;

use crate::Axis;

/// A simple 3D vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Create a new `Vec3` with the given coordinates.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// Generate a random unit `Vec3` inside the unit circle.
    pub fn random_unit(rng: &mut impl Rng) -> Self {
        loop {
            let v = rng.gen::<Vec3>() * 2.0 - 1.0;

            if v.norm2() < 1.0 {
                break v;
            }
        }
    }

    /// `Vec3` with everything set to 0.
    pub const fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

    /// Calculate the distance between two `Vec3`.
    pub fn dist(&self, other: &Vec3) -> f64 {
        self.dist2(other).sqrt()
    }

    /// Calculate the squared distance between two `Vec3`.
    pub fn dist2(&self, other: &Vec3) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)
    }

    /// Calculate the norm or length of this `Vec3`.
    pub fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }

    /// Calculate the squared norm or length of this `Vec3`.
    pub fn norm2(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    /// Normalize this `Vec3` so that its norm is 1 and all its components are
    /// between in [0, 1].
    pub fn normalize(&mut self) {
        *self /= self.norm();
    }

    /// Return a new normalized copy of this `Vec3`.
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    /// Calculate the [dot product][0] between two `Vec3`.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Dot_product
    pub fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// Calculate the [cross product][0] between two `Vec3`.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Cross_product
    pub fn cross(&self, v: &Vec3) -> Self {
        Vec3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }
}

/// [Linear interpolation][0] between two `Vec3`.
///
/// ```rust
/// # use geo::vec3::{lerp, Vec3};
///
/// let o = Vec3::zero();
/// let p = Vec3::new(2.0, 0.0, -6.0);
///
/// assert_eq!(lerp(o, p, 0.0), o);
/// assert_eq!(lerp(o, p, 1.0), p);
/// assert_eq!(lerp(o, p, 0.5), Vec3::new(1.0, 0.0, -3.0));
/// ```
pub fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    a * (1.0 - t) + b * t
}

macro_rules! impl_num_op {
    ($tr:ident, $fn:ident, $op:tt, $assign_tr:ident, $assign_fn:ident) => {
        impl $tr for Vec3 {
            type Output = Vec3;

            fn $fn(self, v: Vec3) -> Self::Output {
                Vec3::new(self.x $op v.x, self.y $op v.y, self.z $op v.z)
            }
        }

        impl $tr<f64> for Vec3 {
            type Output = Vec3;

            fn $fn(self, s: f64) -> Self::Output {
                Vec3::new(self.x $op s, self.y $op s, self.z $op s)
            }
        }

        impl $assign_tr for Vec3 {
            fn $assign_fn(&mut self, v: Vec3) {
                self.x.$assign_fn(v.x);
                self.y.$assign_fn(v.y);
                self.z.$assign_fn(v.z);
            }
        }

        impl $assign_tr<f64> for Vec3 {
            fn $assign_fn(&mut self, s: f64) {
                self.x.$assign_fn(s);
                self.y.$assign_fn(s);
                self.z.$assign_fn(s);
            }
        }
    };
}

impl_num_op!(Add, add, +, AddAssign, add_assign);
impl_num_op!(Sub, sub, -, SubAssign, sub_assign);
impl_num_op!(Mul, mul, *, MulAssign, mul_assign);
impl_num_op!(Div, div, /, DivAssign, div_assign);

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;

        self
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Vec3>>(iter: I) -> Vec3 {
        iter.fold(Vec3::zero(), Add::add)
    }
}

impl Product for Vec3 {
    fn product<I: Iterator<Item = Vec3>>(iter: I) -> Vec3 {
        iter.fold(Vec3::new(1.0, 1.0, 1.0), Mul::mul)
    }
}

impl Distribution<Vec3> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let (x, y, z) = rng.gen();
        Vec3::new(x, y, z)
    }
}

impl Index<Axis> for Vec3 {
    type Output = f64;

    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

#[cfg(test)]
pub fn vec3() -> impl Strategy<Value = Vec3> {
    any::<(f64, f64, f64)>().prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

#[cfg(test)]
pub fn distinct_vec3(
    range: impl Into<proptest::collection::SizeRange>,
) -> impl Strategy<Value = Vec<Vec3>> {
    proptest::collection::hash_set(any::<(i16, i16, i16)>(), range).prop_map(|cs| {
        cs.into_iter()
            .map(|(x, y, z)| Vec3::new(f64::from(x), f64::from(y), f64::from(z)))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math_ops() {
        let v = Vec3::zero();

        assert_eq!(v + Vec3::new(2.0, 1.0, 0.0) * 2.0, Vec3::new(4.0, 2.0, 0.0));
        assert_eq!(
            v - Vec3::new(9.0, -6.0, 3.0) / 3.0,
            Vec3::new(-3.0, 2.0, -1.0)
        );

        assert_eq!(
            (v + 5.0) * Vec3::new(2.0, -1.0, 0.0),
            Vec3::new(10.0, -5.0, 0.0)
        );
        assert_eq!(
            (v - 2.0) / Vec3::new(-2.0, 1.0, 4.0),
            Vec3::new(1.0, -2.0, -0.5)
        );

        assert_eq!(-(v + Vec3::new(1.0, 2.0, 3.0)), Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_dist() {
        let o = Vec3::zero();
        let v2 = Vec3::new(3.0, 4.0, 0.0);

        assert_eq!(o.dist(&v2), 5.0);
        assert_eq!(o.dist2(&v2), 25.0);
    }

    #[test]
    fn test_norm() {
        let v = Vec3::new(0.0, 3.0, 4.0);

        assert_eq!(v.norm(), 5.0);
        assert_eq!(v.norm2(), 25.0);
        assert_eq!(v.normalized().norm(), 1.0);

        let mut nv = v.clone();
        nv.normalize();
        assert_eq!(nv.norm(), 1.0);
    }

    #[test]
    fn test_dot() {
        assert_eq!(Vec3::zero().dot(&Vec3::new(-5.0, 3.0, 1.0)), 0.0);
        assert_eq!(Vec3::new(-5.0, 3.0, 1.0).dot(&Vec3::zero()), 0.0);

        assert_eq!(
            Vec3::new(1.0, 3.0, -5.0).dot(&Vec3::new(4.0, -2.0, -1.0)),
            3.0
        );
    }

    #[test]
    fn test_cross() {
        let right = Vec3::new(1.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);

        assert_eq!(right.cross(&up), forward);
        assert_eq!(up.cross(&right), -forward);
    }

    #[test]
    fn test_sum() {
        assert_eq!(Vec::<Vec3>::new().into_iter().sum::<Vec3>(), Vec3::zero());

        assert_eq!(
            vec![Vec3::new(-10.0, 5.0, 0.0), Vec3::new(8.0, 2.0, -1.0)]
                .into_iter()
                .sum::<Vec3>(),
            Vec3::new(-2.0, 7.0, -1.0)
        );
    }

    #[test]
    fn test_product() {
        assert_eq!(
            vec![Vec3::new(-10.0, 5.0, 0.0), Vec3::new(8.0, 2.0, -1.0)]
                .into_iter()
                .product::<Vec3>(),
            Vec3::new(-80.0, 10.0, 0.0)
        );
    }

    #[test]
    fn test_index() {
        let v = Vec3::new(1.0, 2.0, 3.0);

        assert_eq!(v[Axis::X], 1.0);
        assert_eq!(v[Axis::Y], 2.0);
        assert_eq!(v[Axis::Z], 3.0);
    }
}
