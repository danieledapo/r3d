use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::sphere;
use geo::{Aabb, Vec3};

use crate::{Hit, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct SphereGeometry {
    pub center: Vec3,
    pub radius: f64,
}

impl SphereGeometry {
    pub fn new(center: Vec3, radius: f64) -> Self {
        SphereGeometry { center, radius }
    }
}

impl<'s> Shape<'s> for SphereGeometry {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        sphere::bounding_box(self.center, self.radius)
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.center, self.radius)
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let t = sphere::ray_intersection(self.center, self.radius, ray)?;

        Some(Hit { t, surface: self })
    }
}

impl Surface for SphereGeometry {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        sphere::normal(self.center, pt)
    }
}
