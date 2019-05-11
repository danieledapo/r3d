use geo::plane;
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::{Hit, Object, Surface};

/// An infinite plane.
#[derive(Debug, PartialEq, Clone)]
pub struct Plane {
    pub geom: PlaneGeometry,
    pub material: Material,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PlaneGeometry {
    pub origin: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn new(origin: Vec3, normal: Vec3, material: Material) -> Self {
        Plane {
            geom: PlaneGeometry { origin, normal },
            material,
        }
    }
}

impl Object<'_> for Plane {
    fn material(&self) -> &Material {
        &self.material
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        (self.geom.origin, std::f64::INFINITY)
    }
}

impl Surface for Plane {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        self.geom.normal_at(pt)
    }
}

impl<'s> Shape<'s> for Plane {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        self.geom.intersection(ray)
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

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let t = plane::intersection(self.origin, self.normal, ray)?;

        Some(Hit { t, surface: self })
    }
}
