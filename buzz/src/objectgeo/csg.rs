//! Simple [Constructive Solid Geometry][0] framework.
//!
//! [0]: https://en.wikipedia.org/wiki/Constructive_solid_geometry

use geo::{ray::Ray, sdf::Sdf, spatial_index::Shape, Aabb, Vec3};

use crate::{Hit, Surface};

#[derive(Debug)]
pub struct SdfGeometry {
    sdf: Sdf,
}

impl SdfGeometry {
    pub fn new(sdf: Sdf) -> Self {
        SdfGeometry { sdf }
    }
}

impl Surface for SdfGeometry {
    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.sdf.normal_at(p)
    }
}

impl Shape for SdfGeometry {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        self.sdf.bbox()
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = self.sdf.ray_march(ray, 1000)?;
        Some(Hit::new(t, None))
    }
}
