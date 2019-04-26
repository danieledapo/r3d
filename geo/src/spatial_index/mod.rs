pub mod bvh;
pub mod kdtree;

pub use bvh::Bvh;
pub use kdtree::KdTree;

use crate::ray::Ray;
use crate::{Aabb, Vec3};

use std::ops::Deref;

pub trait Shape: std::fmt::Debug {
    fn bbox(&self) -> Aabb;
    fn intersection(&self, ray: &Ray) -> Option<f64>;
}

impl Shape for Vec3 {
    fn bbox(&self) -> Aabb {
        Aabb::new(*self)
    }

    fn intersection(&self, ray: &Ray) -> Option<f64> {
        let t = ray.t_of(*self)?;

        // we're only interested in intersection on the ray and not its opposite
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

impl<T> Shape for Box<T>
where
    T: Shape + ?Sized,
{
    fn bbox(&self) -> Aabb {
        self.deref().bbox()
    }

    fn intersection(&self, ray: &Ray) -> Option<f64> {
        self.deref().intersection(ray)
    }
}
