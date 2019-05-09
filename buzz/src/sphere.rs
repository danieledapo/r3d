use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::sphere;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::{Hit, Object};

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
}

impl<'a> Object<'a> for Sphere {
    fn material(&self) -> &Material {
        &self.material
    }

    fn normal_at(&self, pt: Vec3) -> Vec3 {
        sphere::normal(self.center, pt)
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.center, self.radius)
    }
}

impl<'s> Shape<'s> for Sphere {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        sphere::bounding_box(self.center, self.radius)
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let t = sphere::ray_intersection(self.center, self.radius, ray)?;

        Some(Hit { t, object: self })
    }
}
