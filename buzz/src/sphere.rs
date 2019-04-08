use geo::ray::Ray;
use geo::sphere;
use geo::vec3::Vec3;

use crate::material::Material;

#[derive(Debug, PartialEq, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }

    pub fn intersection(&self, ray: &Ray) -> Option<f64> {
        sphere::ray_intersection(self.center, self.radius, ray)
    }

    pub fn normal_at(&self, pt: Vec3) -> Vec3 {
        sphere::normal(self.center, pt)
    }
}
