pub mod camera;
pub mod object;
mod renderer;

use std::sync::Arc;

use geo::{
    primitive::polyline::Polyline,
    ray::Ray,
    spatial_index::{Bvh, Intersection, Shape},
};

pub use camera::Camera;
pub use object::*;
pub use renderer::*;

pub trait Object: Shape<Intersection = f64> + std::fmt::Debug + Send + Sync {
    /// Polylines that are part of this Object.
    ///
    /// Note that all paths are considered open if the last point doesn't
    /// exactly match the first one.
    fn paths(&self) -> Vec<Polyline>;
}

#[derive(Debug)]
pub struct Scene {
    objects: Bvh<Arc<dyn Object>>,
}

impl Scene {
    pub fn new(objects: impl IntoIterator<Item = Arc<dyn Object>>) -> Self {
        Self {
            objects: objects.into_iter().collect(),
        }
    }

    pub fn intersection(&self, ray: &Ray) -> Option<(&dyn Object, f64)> {
        self.objects
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.t().partial_cmp(&t1.t()).unwrap())
            .map(|(s, t)| (s.as_ref(), t))
    }
}
