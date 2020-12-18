use geo::{plane, ray::Ray, spatial_index::Shape, Aabb, Vec3};

use crate::{Hit, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct PlaneGeometry {
    pub origin: Vec3,
    pub normal: Vec3,
}

impl PlaneGeometry {
    pub fn new(origin: Vec3, normal: Vec3) -> Self {
        PlaneGeometry { origin, normal }
    }
}

impl Surface for PlaneGeometry {
    fn normal_at(&self, _pt: Vec3) -> Vec3 {
        self.normal
    }
}

impl Shape for PlaneGeometry {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        plane::bbox()
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.origin, std::f64::INFINITY)
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = plane::intersection(self.origin, self.normal, ray)?;
        Some(Hit::new(t, None))
    }
}
