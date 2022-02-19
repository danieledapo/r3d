use geo::{ray::Ray, spatial_index::Shape, Aabb, Triangle, Vec3};

use crate::{material::Material, FacetGeometry, Hit, Object, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct Facet<'a> {
    geom: FacetGeometry,
    material: &'a Material,
    surface_id: usize,
}

impl<'a> Facet<'a> {
    pub fn new(tri: Triangle, material: &'a Material, flat_shading: bool) -> Self {
        Facet {
            geom: FacetGeometry::new(tri, flat_shading),
            material,
            surface_id: 0,
        }
    }
}

impl Object for Facet<'_> {
    fn material(&self) -> &Material {
        self.material
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
