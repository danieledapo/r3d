use geo::{ray::Ray, spatial_index::Shape, Aabb, Triangle, Vec3};

use crate::{Hit, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct FacetGeometry {
    tri: Triangle,
    normal: Vec3,
    flat_shading: bool,
}

impl FacetGeometry {
    pub fn new(tri: Triangle, flat_shading: bool) -> Self {
        let normal = tri.normal();
        FacetGeometry {
            tri,
            flat_shading,
            normal,
        }
    }
}
impl Shape for FacetGeometry {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        self.tri.bbox()
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = self.tri.intersection(ray)?;
        Some(Hit::new(t, None))
    }
}

impl Surface for FacetGeometry {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        if self.flat_shading {
            self.normal
        } else {
            let bary = self.tri.barycentric(&pt).expect("point outside triangle");

            let mut n = self.tri.a * bary.x;
            n += self.tri.b * bary.y;
            n += self.tri.c * bary.z;
            n.normalize();

            n
        }
    }
}
