use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A simple 3D vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Create a new `Vec3` with the given coordinates.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// `Vec3` with everything set to 0.
    pub fn zero() -> Self {
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
}
