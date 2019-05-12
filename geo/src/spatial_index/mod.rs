pub mod bvh;
pub mod kdtree;

pub use bvh::Bvh;
pub use kdtree::KdTree;

use crate::ray::Ray;
use crate::{Aabb, Vec3};

use std::ops::Deref;

/// A `Shape` is a 2d figure that can be inserted in a spatial index. A `Shape`
/// must have a bounding box and provides a way to check for the first
/// intersection with a `Ray`.
pub trait Shape<'s>: std::fmt::Debug {
    /// The type of the `Intersection` returned by `intersection`. Usually it's
    /// `f64` if there's no additional information gathered during the
    /// intersection checking.
    type Intersection: Intersection<'s>;

    /// Calculate the intersection between this `Shape` and a `Ray` returning at
    /// least the parameter `t` that can be used to retrieve the intersection
    /// point.
    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection>;

    /// The bounding box of the `Shape`.
    fn bbox(&self) -> Aabb;

    /// The bounding sphere of this `Shape`. By default it's the bounding sphere
    /// of `bbox()`.
    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.bbox().bounding_sphere()
    }
}

/// Simple trait over an `Intersection`. Can be used to return additional
/// information other than the `t` parameter.
pub trait Intersection<'s> {
    fn t(&self) -> f64;
}

impl Shape<'_> for Vec3 {
    type Intersection = f64;

    fn bbox(&self) -> Aabb {
        Aabb::new(*self)
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = ray.t_of(*self)?;

        // we're only interested in intersection on the ray and not its opposite
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (*self, 0.0)
    }
}

impl Intersection<'_> for f64 {
    fn t(&self) -> f64 {
        *self
    }
}

impl<'s, T> Shape<'s> for Box<T>
where
    T: Shape<'s> + ?Sized + 's,
{
    type Intersection = T::Intersection;

    fn bbox(&self) -> Aabb {
        self.deref().bbox()
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        self.deref().intersection(ray)
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.deref().bounding_sphere()
    }
}
