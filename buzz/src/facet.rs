use geo::mesh::stl;
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::triangle;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::{Hit, Object, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct Facet<'a> {
    geom: FacetGeometry,
    material: &'a Material,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FacetGeometry {
    positions: [Vec3; 3],
    normal: Vec3,
    flat_shading: bool,
}

impl<'a> Facet<'a> {
    pub fn new(tri: stl::StlTriangle, material: &'a Material, flat_shading: bool) -> Self {
        Facet {
            geom: FacetGeometry::new(tri, flat_shading),
            material,
        }
    }
}

impl FacetGeometry {
    pub fn new(tri: stl::StlTriangle, flat_shading: bool) -> Self {
        let normal = geo::triangle::normal(&tri.positions[0], &tri.positions[1], &tri.positions[2]);

        FacetGeometry {
            positions: tri.positions,
            flat_shading,
            normal,
        }
    }
}

impl<'a> Object<'a> for Facet<'a> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.bbox().bounding_sphere()
    }
}

impl<'a> Shape<'a> for Facet<'a> {
    type Intersection = Hit<'a>;

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn intersection(&'a self, ray: &Ray) -> Option<Self::Intersection> {
        self.geom.intersection(ray)
    }
}

impl<'a> Surface for Facet<'a> {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        self.geom.normal_at(pt)
    }
}

impl<'a> Shape<'a> for FacetGeometry {
    type Intersection = Hit<'a>;

    fn bbox(&self) -> Aabb {
        Aabb::new(self.positions[0])
            .expanded(&self.positions[1])
            .expanded(&self.positions[2])
    }

    fn intersection(&'a self, ray: &Ray) -> Option<Self::Intersection> {
        let t = triangle::ray_intersection(
            (self.positions[0], self.positions[1], self.positions[2]),
            ray,
        )?;

        Some(Hit { t, surface: self })
    }
}

impl<'a> Surface for FacetGeometry {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        if self.flat_shading {
            self.normal
        } else {
            let bary = triangle::barycentric(
                (&self.positions[0], &self.positions[1], &self.positions[2]),
                &pt,
            )
            .expect("point outside triangle");

            let mut n = self.positions[0] * bary.x;
            n += self.positions[1] * bary.y;
            n += self.positions[2] * bary.z;
            n.normalize();

            n
        }
    }
}
