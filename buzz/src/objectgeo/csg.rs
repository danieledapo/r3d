//! Simple [Constructive Solid Geometry][0] framework.
//!
//! [0]: https://en.wikipedia.org/wiki/Constructive_solid_geometry

use geo::{
    ray::Ray,
    sdf::{self, Sdf},
    spatial_index::Shape,
    Aabb, Vec3,
};

use crate::{Hit, Surface};

#[derive(Debug)]
pub struct SdfGeometry<S> {
    sdf: S,
}

impl<S: Sdf> SdfGeometry<S> {
    pub fn new(sdf: S) -> Self {
        SdfGeometry { sdf }
    }
}

impl<S: Sdf> Surface for SdfGeometry<S> {
    fn normal_at(&self, p: Vec3) -> Vec3 {
        sdf::normal_at(&self.sdf, p)
    }
}

impl<S: Sdf> Shape for SdfGeometry<S> {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        self.sdf.bbox()
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = sdf::ray_marching(&self.sdf, ray, 1000)?;
        Some(Hit::new(t, None))
    }
}
