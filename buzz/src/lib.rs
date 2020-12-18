#![allow(clippy::useless_let_if_seq)]

pub mod camera;
pub mod material;
pub mod object;
pub mod objectgeo;

mod renderer;

use std::sync::Arc;

use geo::{
    ray::Ray,
    spatial_index::{Bvh, Intersection, Shape},
    Vec3,
};

pub use camera::Camera;
pub use material::Material;
pub use object::*;
pub use objectgeo::*;
pub use renderer::*;

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug)]
pub struct Scene {
    objects: SceneObjects,
    objects_index: Bvh<Arc<dyn Object>>,
    environment: Environment,
}

#[derive(Debug)]
pub struct SceneObjects {
    objects: Vec<Arc<dyn Object>>,
}

/// The `Environment` surrounding the objects in a `Scene`. All the rays that
/// don't hit any objects will hit the environment.
#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    /// The `Environment` is a simple RGB color where each channel is in [0, 1].
    Color(Vec3),

    /// The `Environment` is a simple linear gradient between two RGB colors.
    LinearGradient(Vec3, Vec3),
}

impl Scene {
    /// Create a new `Scene` with the given objects inside the given
    /// `Environment`.
    pub fn new(objects: SceneObjects, environment: Environment) -> Self {
        let objects_index: Bvh<_> = objects.iter().cloned().collect();
        Scene {
            objects,
            objects_index,
            environment,
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection(&self, ray: &Ray) -> Option<(&dyn Object, Hit)> {
        self.objects_index
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.t().partial_cmp(&t1.t()).unwrap())
            .map(|(s, t)| (s.as_ref(), t))
    }

    pub fn surface(&self, id: usize) -> &dyn Object {
        self.objects[id].as_ref()
    }

    /// Return an iterator over all the lights in the `Scene`.
    pub fn lights(&self) -> impl Iterator<Item = &dyn Object> {
        self.objects
            .iter()
            .filter(|o| {
                if let Material::Light { .. } = o.material() {
                    true
                } else {
                    false
                }
            })
            .map(|o| o.as_ref())
    }
}

impl SceneObjects {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }

    pub fn push(&mut self, mut o: impl Object + 'static) {
        o.set_surface_id(self.objects.len());
        self.objects.push(Arc::new(o));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Object>> {
        self.objects.iter()
    }
}

impl std::ops::Index<usize> for SceneObjects {
    type Output = Arc<dyn Object>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.objects[index]
    }
}
