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
    surface_id: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FacetGeometry {
    positions: [Vec3; 3],
    normal: Vec3,
    flat_shading: bool,
}

impl<'a> Facet<'a> {
    pub fn new(vertices: [Vec3; 3], material: &'a Material, flat_shading: bool) -> Self {
        Facet {
            geom: FacetGeometry::new(vertices, flat_shading),
            material,
            surface_id: 0,
        }
    }
}

impl FacetGeometry {
    pub fn new(positions: [Vec3; 3], flat_shading: bool) -> Self {
        let normal = geo::triangle::normal(&positions[0], &positions[1], &positions[2]);

        FacetGeometry {
            positions,
            flat_shading,
            normal,
        }
    }
}

impl Object for Facet<'_> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn set_surface_id(&mut self, sfid: usize) {
        self.surface_id = sfid;
    }
}

impl Shape for Facet<'_> {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let mut h = self.geom.intersection(ray)?;
        h.surface_id = self.surface_id;
        Some(h)
    }
}

impl<'a> Surface for Facet<'a> {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        self.geom.normal_at(pt)
    }
}

impl Shape for FacetGeometry {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        Aabb::new(self.positions[0])
            .expanded(&self.positions[1])
            .expanded(&self.positions[2])
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = triangle::ray_intersection(
            (self.positions[0], self.positions[1], self.positions[2]),
            ray,
        )?;

        Some(Hit::new(t, None))
    }
}

impl Surface for FacetGeometry {
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
