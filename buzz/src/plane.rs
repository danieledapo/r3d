use geo::plane;
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

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

impl<'s> Shape<'s> for PlaneGeometry {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        plane::bbox()
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.origin, std::f64::INFINITY)
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let t = plane::intersection(self.origin, self.normal, ray)?;

        Some(Hit { t, surface: self })
    }
}
