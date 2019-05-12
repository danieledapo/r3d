use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::sphere;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::{Hit, Object, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct Sphere {
    pub geom: SphereGeometry,
    pub material: Material,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SphereGeometry {
    pub center: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Material) -> Self {
        Sphere {
            geom: SphereGeometry { center, radius },
            material,
        }
    }
}

impl Object<'_> for Sphere {
    fn material(&self) -> &Material {
        &self.material
    }
}

impl<'s> Shape<'s> for Sphere {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        self.geom.intersection(ray)
    }
}

impl Surface for Sphere {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        self.geom.normal_at(pt)
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
