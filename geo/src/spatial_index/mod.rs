pub mod bvh;
pub mod kdtree;

pub use bvh::Bvh;
pub use kdtree::KdTree;

use crate::ray::Ray;
use crate::{Aabb, Vec3};

use std::ops::Deref;

pub trait Shape<'s>: std::fmt::Debug {
    type Intersection: Intersection<'s>;

    fn bbox(&self) -> Aabb;
    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection>;
}

pub trait Intersection<'s> {
    fn t(&self) -> f64;
}

impl<'s> Shape<'s> for Vec3 {
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
}

impl<'s> Intersection<'s> for f64 {
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
}
