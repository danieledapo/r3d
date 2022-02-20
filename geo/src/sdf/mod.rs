//! A collection of common [Signed Distance Functions][0].
//!
//! [0]: http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
//!

use crate::{mat4::Mat4, ray::Ray, Aabb, Vec3};

mod combinations;
pub mod primitives;

pub use self::combinations::{Difference, Intersection, Transformed, Union};
pub use primitives::*;

/// An SDF is a function that when called with a point it returns the distance
/// from that point to the object itself.
///
/// In particular, that distance will be negative when the point is inside the
/// object, positive when the point is outside and almost 0 when the point is on
/// the boundary of the object.
///
/// This representation, besides being quite lightweight, allows easy
/// composition of objects via simple boolean operations (or, and, sub).
pub trait Sdf: std::fmt::Debug + Sized {
    /// Return the distance from the given point to the object.
    ///
    /// The distance will be:
    /// - negative if the point is inside the object
    /// - positive if the point is outside the object
    /// - 0 (or almost 0) if the point is exactly on the boundary of the object
    fn dist(&self, p: &Vec3) -> f64;

    /// Return the bounding box of the Sdf.
    fn bbox(&self) -> Aabb;

    /// Return the union (aka or) between this Sdf and another one.
    fn union<S: Sdf>(self, other: S) -> Union<Self, S> {
        Union {
            left: self,
            right: other,
        }
    }

    /// Return the intersection (aka and) between this Sdf and another one.
    fn intersection<S: Sdf>(self, other: S) -> Intersection<Self, S> {
        Intersection {
            left: self,
            right: other,
        }
    }

    /// Return the difference (aka -) between this Sdf and another one.
    fn difference<S: Sdf>(self, other: S) -> Difference<Self, S> {
        Difference {
            left: self,
            right: other,
        }
    }

    /// Return the current Sdf transformed with the given transformation matrix.
    ///
    /// Note that the given transformation should be continuous and should not
    /// introduce discontinuities.
    fn transformed(self, xform: Mat4) -> Transformed<Self> {
        let inverse_matrix = xform.inverse();
        Transformed {
            sdf: self,
            matrix: xform,
            inverse_matrix,
        }
    }
}

/// Calculate the normal at the given point on the Sdf.
///
/// Note that the point is assumed to be on the surface of the Sdf and no checks
/// are made in this regard.
pub fn normal_at(s: &impl Sdf, p: Vec3) -> Vec3 {
    let e = 0.000001;
    let Vec3 { x, y, z } = p;
    let n = Vec3::new(
        s.dist(&Vec3::new(x + e, y, z)) - s.dist(&Vec3::new(x - e, y, z)),
        s.dist(&Vec3::new(x, y + e, z)) - s.dist(&Vec3::new(x, y - e, z)),
        s.dist(&Vec3::new(x, y, z + e)) - s.dist(&Vec3::new(x, y, z - e)),
    );
    n.normalized()
}

/// Calculate the intersection between a given Ray and an Sdf.
///
/// The algorithm used is a form of [Ray marching][0] which basically queries
/// the Sdf along the ray multiple times until the distance becomes 0 (i.e. we
/// touched the surface of the object). If the distance never reaches 0 then
/// there's no intersection.
///
/// The steps parameter decides the maximum numer of queries we can permorm
/// before giving up and returning a no intersection.
///
/// Returns either the t parameter of the intersection or none if no
/// intersection was found.
///
/// [0]: https://en.wikipedia.org/wiki/Volume_ray_casting#Ray_Marching
pub fn ray_marching(sdf: &impl Sdf, ray: &Ray, steps: usize) -> Option<f64> {
    let epsilon = 0.00001;
    let jump_size = 0.001;

    let (t1, t2) = sdf.bbox().ray_intersection(ray)?;
    if t2 < t1 || t2 < 0.0 {
        return None;
    }

    let mut t = t1.max(0.0001);
    let mut jump = true;

    // ray marching
    for _ in 0..steps {
        let mut d = sdf.dist(&ray.point_at(t));

        if jump && d < 0.0 {
            t -= jump_size;
            jump = false;
            continue;
        }

        if d < epsilon {
            return Some(t);
        }

        if jump && d < jump_size {
            d = jump_size;
        }

        t += d;

        if t > t2 {
            break;
        }
    }

    None
}
