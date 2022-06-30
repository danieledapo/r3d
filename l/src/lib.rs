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

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug)]
pub struct Scene {
    objects: Bvh<Arc<dyn Object>>,
}

/// An `Object` that can be rendered.
pub trait Object: Shape<Intersection = f64> + std::fmt::Debug + Send + Sync {
    /// Polylines that are part of this Object.
    ///
    /// Note that all paths are considered open if the last point doesn't
    /// exactly match the first one.
    fn paths(&self) -> Vec<Polyline>;
}

impl Scene {
    /// Create a new `Scene` with the given objects.
    pub fn new(objects: impl IntoIterator<Item = Arc<dyn Object>>) -> Self {
        Self {
            objects: objects.into_iter().collect(),
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection t
    /// parameter) to the ray.
    pub fn intersection(&self, ray: &Ray) -> Option<(&dyn Object, f64)> {
        self.objects
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.t().total_cmp(&t1.t()))
            .map(|(s, t)| (s.as_ref(), t))
    }
}
