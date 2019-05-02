use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::Object;

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

impl Object for Plane {
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

impl Shape for Plane {
    fn bbox(&self) -> Aabb {
        use std::f64::{INFINITY, NEG_INFINITY};

        Aabb::new(Vec3::new(INFINITY, INFINITY, INFINITY)).expanded(&Vec3::new(
            NEG_INFINITY,
            NEG_INFINITY,
            NEG_INFINITY,
        ))
    }

    fn intersection(&self, ray: &Ray) -> Option<f64> {
        let d = self.normal.dot(&ray.dir);
        if d.abs() < 1e-6 {
            return None;
        }

        let a = self.origin - ray.origin;
        let t = a.dot(&self.normal) / d;
        if t < 1e-6 {
            return None;
        }

        Some(t)
    }
}
