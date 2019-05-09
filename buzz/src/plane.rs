use geo::plane;
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::{Hit, Object};

/// An infinite plane.
#[derive(Debug, PartialEq, Clone)]
pub struct Plane {
    pub origin: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Plane {
    pub fn new(origin: Vec3, normal: Vec3, material: Material) -> Self {
        Plane {
            origin,
            material,
            normal,
        }
    }
}

impl<'a> Object<'a> for Plane {
    fn material(&self) -> &Material {
        &self.material
    }

    fn normal_at(&self, _pt: Vec3) -> Vec3 {
        self.normal
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.origin, std::f64::INFINITY)
    }
}

impl<'a> Shape<'a> for Plane {
    type Intersection = Hit<'a>;

    fn bbox(&self) -> Aabb {
        plane::bbox()
    }

    fn intersection(&'a self, ray: &Ray) -> Option<Self::Intersection> {
        let t = plane::intersection(self.origin, self.normal, ray)?;

        Some(Hit { t, object: self })
    }
}
