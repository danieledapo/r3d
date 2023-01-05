//! A collection of common [Signed Distance Functions][0].
//!
//! [0]: http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
//!

use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, Mul, Sub},
    sync::Arc,
};

use crate::{mat4::Mat4, ray::Ray, v3, Aabb, Vec3};

pub mod primitives;
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
#[derive(Clone)]
pub struct Sdf {
    /// Return the distance from the given point to the object.
    ///
    /// The distance will be:
    /// - negative if the point is inside the object
    /// - positive if the point is outside the object
    /// - 0 (or almost 0) if the point is exactly on the boundary of the object
    dist: Arc<dyn Fn(&Vec3) -> f64 + Send + Sync + 'static>,

    /// The bounding box of the Sdf.
    bbox: Aabb,
}

impl Sdf {
    /// Create the Sdf from the given function and with the given visibility
    /// bounding box.
    pub fn from_fn(bbox: Aabb, dist: impl Fn(&Vec3) -> f64 + Send + Sync + 'static) -> Self {
        Self {
            dist: Arc::new(dist),
            bbox,
        }
    }

    /// Calculate the distance to the given point from the Sdf.
    pub fn dist(&self, p: &Vec3) -> f64 {
        (self.dist)(p)
    }

    /// Return the bounding box of the Sdf
    pub fn bbox(&self) -> Aabb {
        self.bbox.clone()
    }

    /// Calculate the normal at the given point on the Sdf.
    ///
    /// Note that the point is assumed to be on the surface of the Sdf and no
    /// checks are made in this regard.
    pub fn normal_at(&self, p: Vec3) -> Vec3 {
        let e = 0.000001;
        let Vec3 { x, y, z } = p;
        let n = v3(
            self.dist(&v3(x + e, y, z)) - self.dist(&v3(x - e, y, z)),
            self.dist(&v3(x, y + e, z)) - self.dist(&v3(x, y - e, z)),
            self.dist(&v3(x, y, z + e)) - self.dist(&v3(x, y, z - e)),
        );
        n.normalized()
    }

    /// Calculate the intersection between a given Ray and an Sdf.
    ///
    /// The algorithm used is a form of [Ray marching][0] which basically
    /// queries the Sdf along the ray multiple times until the distance becomes
    /// 0 (i.e. we touched the surface of the object). If the distance never
    /// reaches 0 then there's no intersection.
    ///
    /// The steps parameter decides the maximum number of queries we can perform
    /// before giving up and returning a no intersection.
    ///
    /// Returns either the t parameter of the intersection or none if no
    /// intersection was found.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Volume_ray_casting#Ray_Marching
    pub fn ray_march(&self, ray: &Ray, steps: usize) -> Option<f64> {
        let epsilon = 0.00001;

        let (t1, t2) = self.bbox.ray_intersection(ray)?;
        if t2 < t1 || t2 < 0.0 {
            return None;
        }

        let mut t = t1.max(0.0001);

        // ray marching
        for _ in 0..steps {
            let d = self.dist(&ray.point_at(t));
            if d < epsilon {
                return Some(t);
            }

            t += d;

            if t > t2 {
                break;
            }
        }

        None
    }

    /// Return a shape that's the surface enlarged by the given thickness.
    ///
    /// All interiors beyond thickness are removed.
    pub fn shell(self, thickness: f64) -> Self {
        let b = Aabb::new(self.bbox.min() - thickness).expanded(self.bbox.max() + thickness);
        Self::from_fn(b, move |p| self.dist(p).abs() - thickness)
    }

    /// Round the SDF with the given radius.
    pub fn round(self, radius: f64) -> Self {
        let b = Aabb::new(self.bbox.min() - radius).expanded(self.bbox.max() + radius);
        Self::from_fn(b, move |p| self.dist(p) - radius)
    }
}

impl Add<Vec3> for Sdf {
    type Output = Self;

    fn add(self, delta: Vec3) -> Self::Output {
        Self::from_fn(self.bbox.translated(delta), move |p| {
            self.dist(&(*p - delta))
        })
    }
}

impl BitOr<Sdf> for Sdf {
    type Output = Self;

    fn bitor(self, rhs: Sdf) -> Self::Output {
        Self::from_fn(self.bbox.union(&rhs.bbox), move |p| {
            let ld = self.dist(p);
            let rd = rhs.dist(p);
            f64::min(ld, rd)
        })
    }
}

impl BitAnd<Sdf> for Sdf {
    type Output = Self;

    fn bitand(self, rhs: Sdf) -> Self::Output {
        Self::from_fn(
            self.bbox
                .intersection(&rhs.bbox)
                .unwrap_or_else(|| Aabb::new(Vec3::zero())),
            move |p| {
                let ld = self.dist(p);
                let rd = rhs.dist(p);
                f64::max(ld, rd)
            },
        )
    }
}

impl Sub<Self> for Sdf {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_fn(self.bbox.clone(), move |p| {
            let ld = self.dist(p);
            let rd = rhs.dist(p);

            f64::max(ld, -rd)
        })
    }
}

impl Mul<Mat4> for Sdf {
    type Output = Self;

    fn mul(self, mat: Mat4) -> Self::Output {
        let inverse_matrix = mat.inverse();
        let bbox = self.bbox.clone() * &mat;
        Sdf::from_fn(bbox, move |&p| {
            let q = p * &inverse_matrix;
            self.dist(&q)
        })
    }
}

impl Debug for Sdf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sdf").field("bbox", &self.bbox).finish()
    }
}
