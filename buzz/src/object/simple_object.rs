use geo::{ray::Ray, spatial_index::Shape, Aabb, Vec3};

use crate::{material::Material, Hit, Object, Surface};

#[derive(Debug)]
pub struct SimpleObject<S> {
    geom: S,
    material: Material,
    surface_id: usize,
}

impl<G> SimpleObject<G> {
    pub fn new(geom: G, material: Material) -> Self {
        SimpleObject {
            geom,
            material,
            surface_id: 0,
        }
    }
}

impl<S> Object for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Surface + Sync + Send,
{
    fn material(&self) -> &Material {
        &self.material
    }

    fn set_surface_id(&mut self, id: usize) {
        self.surface_id = id;
    }
}

impl<S> Surface for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Surface + Sync + Send,
{
    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.geom.normal_at(p)
    }
}

impl<S> Shape for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Sync + Send,
{
    type Intersection = Hit;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let mut h = self.geom.intersection(ray)?;
        h.surface_id = self.surface_id;
        Some(h)
    }

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.geom.bounding_sphere()
    }
}
